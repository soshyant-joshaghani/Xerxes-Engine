-- Initial user table. Same shape as Fast's Alembic revision 001.
CREATE TABLE IF NOT EXISTS "user" (
    id uuid PRIMARY KEY,
    email varchar(255) NOT NULL,
    is_active boolean NOT NULL,
    is_superuser boolean NOT NULL,
    full_name varchar(255),
    hashed_password varchar NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ix_user_email ON "user" (email);
