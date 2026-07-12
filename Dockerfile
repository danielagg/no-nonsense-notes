# ── Stage 1: build ────────────────────────────────────────────────
FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates crates

RUN cargo build --release --bin no-nonsense-notes-server

# ── Stage 2: runtime ──────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/sh app

WORKDIR /app

COPY --from=builder /app/target/release/no-nonsense-notes-server .

RUN chown -R app:app /app

USER app

ENV PORT=8080
ENV DATABASE_URL=server.db

EXPOSE 8080

CMD ["./no-nonsense-notes-server"]
