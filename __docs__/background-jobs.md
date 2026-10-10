# Background jobs

Jobs use a Redis list (`foxg:jobs`) and a Redis list worker (`backend/src/bin/worker.rs`). The protocol is in [CONTRACT.md](../../../CONTRACT.md). There is no second queue library.

- Enqueue from a service after the database write succeeds. Do not run long work inside the request.
- The worker pops with `BRPOP`, runs the task by name, and re-pushes a failed task up to three times.
- Register a task in the worker's task table. `ping` is the example.
- `POST /api/v1/private/jobs/ping?message=hi` enqueues a ping (local only). A missing Redis returns 503.

The full profile starts Redis and the worker. Slim mode omits both. Production always includes the worker.
