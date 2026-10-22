#!/usr/bin/env python3
"""FoxG wire-contract test. Runs against any backend that implements CONTRACT.md.

    python contract_test.py                         # http://localhost:8000, admin@example.com / Admin@1234
    python contract_test.py --base http://api.localhost
    python contract_test.py --local                 # also check the local-only /private routes

Standard library only. Exit code 0 when every check passes.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
import uuid

UUID_RE = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
USER_KEYS = {"id", "email", "is_active", "is_superuser", "full_name"}


class Client:
    def __init__(self, base: str, prefix: str):
        self.base = base.rstrip("/")
        self.prefix = prefix

    def call(self, method, path, *, token=None, json_body=None, form=None, raw=False):
        url = f"{self.base}{path}" if raw else f"{self.base}{self.prefix}{path}"
        headers = {}
        data = None
        if token:
            headers["Authorization"] = f"Bearer {token}"
        if json_body is not None:
            data = json.dumps(json_body).encode()
            headers["Content-Type"] = "application/json"
        if form is not None:
            data = urllib.parse.urlencode(form).encode()
            headers["Content-Type"] = "application/x-www-form-urlencoded"
        req = urllib.request.Request(url, data=data, method=method, headers=headers)
        try:
            with urllib.request.urlopen(req, timeout=20) as res:
                status, body, hdrs = res.status, res.read(), res.headers
        except urllib.error.HTTPError as err:
            status, body, hdrs = err.code, err.read(), err.headers
        parsed = None
        if body:
            try:
                parsed = json.loads(body)
            except ValueError:
                parsed = body.decode("utf-8", "replace")
        return status, parsed, hdrs


class Report:
    def __init__(self):
        self.failed = 0
        self.passed = 0

    def check(self, name, ok, detail=""):
        if ok:
            self.passed += 1
            print(f"  ok    {name}")
        else:
            self.failed += 1
            print(f"  FAIL  {name}" + (f"  -> {detail}" if detail else ""))


def detail_ok(body):
    """`detail` is a string; FastAPI validation errors may use a list of {msg}."""
    return isinstance(body, dict) and (
        isinstance(body.get("detail"), str)
        or (isinstance(body.get("detail"), list) and all(isinstance(i, dict) and "msg" in i for i in body["detail"]))
    )


def run(args) -> int:
    c = Client(args.base, args.prefix)
    r = Report()
    short = lambda b: json.dumps(b)[:160] if not isinstance(b, str) else b[:160]

    print("system")
    s, b, _ = c.call("GET", "/utils/health-check")
    r.check("GET /utils/health-check -> true", s == 200 and b is True, f"{s} {short(b)}")
    for path in ("/docs", "/sdoc"):
        s, _, _ = c.call("GET", path, raw=True)
        r.check(f"GET {path} -> 200", s == 200, str(s))
    s, b, _ = c.call("GET", "/openapi.json")
    r.check("GET /openapi.json -> json", s == 200 and isinstance(b, dict) and "paths" in b, f"{s}")

    print("login")
    s, b, _ = c.call("POST", "/base/login/access-token", form={"username": args.email, "password": "definitely-wrong"})
    r.check("wrong password -> 400 detail", s == 400 and isinstance(b, dict) and b.get("detail") == "Incorrect email or password", f"{s} {short(b)}")
    s, b, _ = c.call("POST", "/base/login/access-token", form={"username": args.email, "password": args.password})
    ok = s == 200 and isinstance(b, dict) and isinstance(b.get("access_token"), str) and str(b.get("token_type", "")).lower() == "bearer"
    r.check("login -> {access_token, token_type:bearer}", ok, f"{s} {short(b)}")
    if not ok:
        print("cannot continue without a token")
        return 1
    admin = b["access_token"]

    s, b, _ = c.call("GET", "/base/login/me", token=admin)
    r.check("GET /base/login/me -> UserPublic", s == 200 and isinstance(b, dict) and set(b) == USER_KEYS and UUID_RE.match(str(b.get("id"))) and b["is_superuser"] is True, f"{s} {short(b)}")
    admin_id = b.get("id") if isinstance(b, dict) else None

    s, b, h = c.call("GET", "/base/login/me")
    r.check("no token -> 401 Not authenticated", s == 401 and isinstance(b, dict) and b.get("detail") == "Not authenticated", f"{s} {short(b)}")
    s, b, h = c.call("GET", "/base/login/me", token="not-a-jwt")
    r.check("bad token -> 401 Could not validate credentials", s == 401 and isinstance(b, dict) and b.get("detail") == "Could not validate credentials", f"{s} {short(b)}")
    r.check("401 has WWW-Authenticate: Bearer", str(h.get("WWW-Authenticate", "")).lower().startswith("bearer"), str(h.get("WWW-Authenticate")))

    print("users (superuser)")
    email = f"contract-{uuid.uuid4().hex[:10]}@example.com"
    pw = "Contract#1234"
    s, b, _ = c.call("POST", "/base/users/admin", token=admin, json_body={"email": email, "password": pw, "full_name": "Contract User"})
    r.check("POST /base/users/admin -> UserPublic", s == 200 and isinstance(b, dict) and set(b) == USER_KEYS and b["email"] == email and b["is_superuser"] is False and b["is_active"] is True, f"{s} {short(b)}")
    user_id = b.get("id") if isinstance(b, dict) else None
    s, b, _ = c.call("POST", "/base/users/admin", token=admin, json_body={"email": email, "password": pw})
    r.check("duplicate email -> 400 detail", s == 400 and detail_ok(b), f"{s} {short(b)}")
    s, b, _ = c.call("GET", "/base/users/admin?skip=0&limit=100", token=admin)
    r.check("GET /base/users/admin -> {data,count}", s == 200 and isinstance(b, dict) and isinstance(b.get("data"), list) and isinstance(b.get("count"), int) and b["count"] >= 2, f"{s} {short(b)}")
    if isinstance(b, dict) and isinstance(b.get("data"), list) and b["data"]:
        r.check("list items are UserPublic", all(set(u) == USER_KEYS for u in b["data"]))
        emails = [u["email"] for u in b["data"]]
        r.check("list ordered by email", emails == sorted(emails), str(emails[:4]))

    s, b, _ = c.call("POST", "/base/login/access-token", form={"username": email, "password": pw})
    r.check("new user can log in", s == 200 and isinstance(b, dict) and "access_token" in b, f"{s} {short(b)}")
    user = b.get("access_token") if isinstance(b, dict) else None

    if user:
        s, b, _ = c.call("GET", "/base/users/admin", token=user)
        r.check("non-superuser list -> 403 privileges", s == 403 and isinstance(b, dict) and b.get("detail") == "The user doesn't have enough privileges", f"{s} {short(b)}")
        s, b, _ = c.call("GET", f"/base/users/{user_id}/admin", token=user)
        r.check("user reads self -> 200", s == 200 and isinstance(b, dict) and b.get("id") == user_id, f"{s} {short(b)}")
        s, b, _ = c.call("GET", f"/base/users/{admin_id}/admin", token=user)
        r.check("user reads other -> 403", s == 403 and detail_ok(b), f"{s} {short(b)}")
        s, b, _ = c.call("PATCH", f"/base/users/{user_id}/admin", token=user, json_body={"full_name": "x"})
        r.check("non-superuser patch -> 403", s == 403 and detail_ok(b), f"{s} {short(b)}")

    s, b, _ = c.call("PATCH", f"/base/users/{user_id}/admin", token=admin, json_body={"full_name": "Renamed"})
    r.check("PATCH user -> UserPublic", s == 200 and isinstance(b, dict) and b.get("full_name") == "Renamed", f"{s} {short(b)}")
    s, b, _ = c.call("PATCH", f"/base/users/{uuid.uuid4()}/admin", token=admin, json_body={"full_name": "x"})
    r.check("PATCH unknown user -> 404", s == 404 and detail_ok(b), f"{s} {short(b)}")
    s, b, _ = c.call("GET", f"/base/users/{uuid.uuid4()}/admin", token=admin)
    r.check("GET unknown user -> 404", s == 404 and detail_ok(b), f"{s} {short(b)}")
    s, b, _ = c.call("DELETE", f"/base/users/{admin_id}/admin", token=admin)
    r.check("delete self -> 403", s == 403 and detail_ok(b), f"{s} {short(b)}")

    print("users (cleanup)")
    if user_id:
        s, b, _ = c.call("DELETE", f"/base/users/{user_id}/admin", token=admin)
        r.check("DELETE user -> {message}", s == 200 and isinstance(b, dict) and isinstance(b.get("message"), str), f"{s} {short(b)}")
        s, b, _ = c.call("POST", "/base/login/access-token", form={"username": email, "password": pw})
        r.check("deleted user cannot log in", s == 400, f"{s} {short(b)}")

    if args.local:
        print("private (local only)")
        s, b, _ = c.call("GET", "/private/ping")
        r.check("GET /private/ping", s == 200 and isinstance(b, dict) and b.get("message") == "private ok", f"{s} {short(b)}")
        pe = f"private-{uuid.uuid4().hex[:8]}@example.com"
        s, b, _ = c.call("POST", "/private/users", json_body={"email": pe, "password": "Private#1234", "full_name": "P"})
        r.check("POST /private/users -> UserPublic", s == 200 and isinstance(b, dict) and set(b) == USER_KEYS, f"{s} {short(b)}")
        if isinstance(b, dict) and b.get("id"):
            c.call("DELETE", f"/base/users/{b['id']}/admin", token=admin)
        if args.jobs:
            s, b, _ = c.call("POST", "/private/jobs/ping?message=contract")
            r.check("POST /private/jobs/ping -> {job_id,message}", s == 200 and isinstance(b, dict) and "job_id" in b and b.get("message") == "contract", f"{s} {short(b)}")

    print(f"\n{r.passed} passed, {r.failed} failed")
    return 1 if r.failed else 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--base", default="http://localhost:8000", help="API origin (no /api/v1)")
    p.add_argument("--prefix", default="/api/v1")
    p.add_argument("--email", default="admin@example.com")
    p.add_argument("--password", default="Admin@1234")
    p.add_argument("--local", action="store_true", help="also check /private routes (ENVIRONMENT=local)")
    p.add_argument("--jobs", action="store_true", help="with --local: enqueue a ping job (needs Redis)")
    return run(p.parse_args())


if __name__ == "__main__":
    sys.exit(main())
