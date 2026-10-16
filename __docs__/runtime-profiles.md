# Runtime profiles

| Profile | Command | Redis | Worker |
|---------|---------|-------|--------|
| Full | `backend run dev` / `backend run prod` | yes | yes |
| Slim | `backend run dev --slim` / `backend run prod --slim` | no | no |

Slim is a supported lightweight mode for CRUD and auth work. A production VM runs full unless it is started with `--slim`.
