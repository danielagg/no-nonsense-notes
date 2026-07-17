# Server crate

`no-nonsense-notes-server` is the Axum service used by every client. It handles
account authentication and stores the per-account update log exchanged over
WebSockets.

## Responsibilities

- Email/password signup and signin
- Opaque authentication tokens
- WebSocket authentication, push, pull, and update notifications
- Server-owned SQLite schema and migrations
- Swagger UI and OpenAPI output

## Run

Set `CORS_ORIGIN` in the environment or a repository-root `.env` file, then run:

```sh
cargo run -p no-nonsense-notes-server
```

Optional settings:

- `PORT` or `LISTEN_ADDR` controls the listener; the default is `3000`.
- `DATABASE_URL` controls the SQLite path; the default is `server.db`.

With the server running:

- API: `http://localhost:3000`
- Swagger UI: `http://localhost:3000/swagger-ui`
- OpenAPI: `http://localhost:3000/api-docs/openapi.json`

## Tests and migrations

```sh
cargo test -p no-nonsense-notes-server
```

Add migrations as numbered SQL files under `src/storage/migrations/`. Server
migrations are intentionally separate from client/core migrations.
