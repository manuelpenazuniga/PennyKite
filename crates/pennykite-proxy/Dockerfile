# syntax=docker/dockerfile:1

FROM rust:1.94.1-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p pennykite-proxy

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl libssl3 \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --create-home --shell /usr/sbin/nologin pennykite \
  && mkdir -p /app/policy /data \
  && chown -R pennykite:pennykite /app /data

COPY --from=builder /app/target/release/pennykite-proxy /usr/local/bin/pennykite-proxy
COPY policy /app/policy

USER pennykite
EXPOSE 8787

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -fsS "http://127.0.0.1:${PORT:-8787}/health" >/dev/null || exit 1

CMD ["/bin/sh", "-c", "exec /usr/local/bin/pennykite-proxy --listen 0.0.0.0:${PORT:-8787} --policy ${PENNYKITE_POLICY:-/app/policy/examples/conservative.yaml} --upstream ${PENNYKITE_UPSTREAM:?set PENNYKITE_UPSTREAM} --db ${DATABASE_PATH:-/data/pennykite.db} --private-key ${KITE_PRIVATE_KEY:?set KITE_PRIVATE_KEY} --usdc-contract ${USDC_CONTRACT:-0x036CbD53842c5426634e7929541eC2318f3dCF7e}"]
