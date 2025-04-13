# Relay

For information on what the Relay is, refer to the [TacoQ Documentation](https://www.tacodivision.com/quickstart/core-concepts)

# Build

## With a database (recommended)

To build the Relay using a database (which you should if you are a developer),
it's a slightly more involved process:

1. Run a Postgres instance using Docker. You can use `dev/docker-compose.yml`
   and run `docker compose up -d postgres`.
2. Apply the migrations. Set `DATABASE_URL` to
   `postgresql://user:password@localhost:5432/tacoq` and run 
   `cargo sqlx migrate run` to apply the database migrations.
3. Run `cargo build`.

## Without a database

To build the Relay from source without an active database connection, follow 
the following steps:

1. Set the environment variable `SQLX_OFFLINE` to `true` to eliminate SQLX
   compiler errors.
2. Run `cargo build`.