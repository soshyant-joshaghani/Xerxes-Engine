# On-VM / local-prod scripts (run inside the project tree)

These bash/bat helpers live in the cloned repo so production can start without the
laptop CLI. Prefer **`xerxes-ctrl`** from your machine when possible.

| Script | CLI equivalent |
|--------|----------------|
| `start-prod.*` | `xerxes-ctrl start` (SSH) or `backend run prod` (local; `--slim` is passed through) |
| `stop-prod.*` | `stop` / `backend stop prod` |
| `reset-prod.*` | `reset` / `backend purge prod` |
| `backup-acme.*` / `restore-acme.*` | `backup-acme` / `restore-acme` / `prod …` |
| `prune-docker-build.*` | `prod prune-build` |
| `setup-ubuntu.sh` | Prefer `xerxes-ctrl setup` from laptop |

`servers.json` points `start_cmd` / `stop_cmd` at these scripts.
