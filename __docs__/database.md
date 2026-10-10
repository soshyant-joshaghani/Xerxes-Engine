# Database

PostgreSQL 18. The schema is the one Fast's Alembic creates (`"user"`), so one database works under any FoxG backend.

SQL files live in `backend/migrations`. The API applies them in name order when it starts and records each in `schema_migrations`. Every statement is `IF NOT EXISTS`, so the API can start against a database Fast already built.

Adminer: http://adminer.localhost, server `db`, port 5432, credentials from `.env`. From the host or an IDE use `localhost:5432`.

After a Postgres volume change: `backend purge dev --only infra`, then `backend run dev --only infra`.
