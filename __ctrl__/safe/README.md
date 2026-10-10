# safe/ — keys, addresses, prod env (local only)

| Pattern | Purpose |
|---------|---------|
| `*-privatekey.pem` | SSH private key |
| `*-address.txt` | VM IP / hostname (first line) |
| `*-env.env` | Production secrets → uploaded as `~/projects/xerxes/.env` |

| Files | Server id |
|-------|-----------|
| `ar-xerxes-bamdad-*` | `xerxes` |

Copy the `*.example` stubs, drop the `.example` suffix, and fill real values.

`*.pem`, `*.env`, `*-address.txt` are gitignored.

Upload env to VM:

```bat
xerxes-ctrl.bat env
```

That copies `safe/ar-xerxes-bamdad-env.env` → `~/projects/xerxes/.env`.
