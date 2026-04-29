FROM rust:1.95.0-bookworm AS rust-build

WORKDIR /usr/local/src/oxid

RUN cargo install sqlx-cli --no-default-features --features postgres

COPY Cargo.toml Cargo.lock ./
COPY libs/args/Cargo.toml libs/args/Cargo.toml
COPY libs/auth/Cargo.toml libs/auth/Cargo.toml
COPY libs/common/Cargo.toml libs/common/Cargo.toml
COPY libs/core/Cargo.toml libs/core/Cargo.toml
COPY libs/handlers/Cargo.toml libs/handlers/Cargo.toml
COPY libs/handlers-organization/Cargo.toml libs/handlers-organization/Cargo.toml
COPY libs/iam/Cargo.toml libs/iam/Cargo.toml
COPY libs/macros/Cargo.toml libs/macros/Cargo.toml
COPY libs/rate-limit/Cargo.toml libs/rate-limit/Cargo.toml
COPY libs/server/Cargo.toml libs/server/Cargo.toml

COPY apps/api/Cargo.toml apps/api/Cargo.toml

RUN \
  mkdir -p libs/args/src libs/auth/src libs/auth/benches libs/common/src libs/core/src libs/handlers/src libs/handlers/benches libs/handlers-organization/src libs/iam/src libs/macros/src libs/rate-limit/src libs/rate-limit/benches libs/server/src apps/api/src && \
  touch libs/args/src/lib.rs libs/auth/src/lib.rs libs/common/src/lib.rs libs/core/src/lib.rs libs/handlers/src/lib.rs libs/handlers-organization/src/lib.rs libs/iam/src/lib.rs libs/macros/src/lib.rs libs/rate-limit/src/lib.rs libs/server/src/lib.rs && \
  echo "fn main() {}" > apps/api/src/main.rs && \
  echo "fn main() {}" > libs/auth/benches/token_decode.rs && \
  echo "fn main() {}" > libs/handlers/benches/rate_limit_headers.rs && \
  echo "fn main() {}" > libs/rate-limit/benches/key_format.rs && \
  cargo build --release

COPY libs/args/src/ libs/args/src/
COPY libs/auth/src/ libs/auth/src/
COPY libs/auth/benches/ libs/auth/benches/
COPY libs/common/src/ libs/common/src/
COPY libs/core/src/ libs/core/src/
COPY libs/handlers/src/ libs/handlers/src/
COPY libs/handlers/benches/ libs/handlers/benches/
COPY libs/handlers-organization/src/ libs/handlers-organization/src/
COPY libs/iam/src/ libs/iam/src/
COPY libs/macros/src/ libs/macros/src/
COPY libs/rate-limit/src/ libs/rate-limit/src/
COPY libs/rate-limit/benches/ libs/rate-limit/benches/
COPY libs/server/src/ libs/server/src/

COPY apps/api/src/ apps/api/src/

COPY .sqlx .sqlx

COPY migrations/ migrations/

RUN \
    touch libs/args/src/lib.rs && \
    touch libs/auth/src/lib.rs && \
    touch libs/common/src/lib.rs && \
    touch libs/core/src/lib.rs && \
    touch libs/handlers/src/lib.rs && \
    touch libs/handlers-organization/src/lib.rs && \
    touch libs/iam/src/lib.rs && \
    touch libs/macros/src/lib.rs && \
    touch libs/rate-limit/src/lib.rs && \
    touch libs/server/src/lib.rs && \
    cargo build --release

FROM debian:bookworm-slim AS runtime

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates=20230311+deb12u1 \
    libssl3=3.0.17-1~deb12u2 && \
    rm -rf /var/lib/apt/lists/* && \
    addgroup \
    --system \
    --gid 1000 \
    oxid && \
    adduser \
    --system \
    --no-create-home \
    --disabled-login \
    --uid 1000 \
    --gid 1000 \
    oxid

USER oxid

FROM runtime AS api

COPY --from=rust-build /usr/local/src/oxid/target/release/api /usr/local/bin/api
COPY --from=rust-build /usr/local/src/oxid/migrations /usr/local/oxid/migrations
COPY --from=rust-build /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT [ "api" ]


FROM node:24.13.0-alpine AS webapp-deps

WORKDIR /usr/local/src/oxid

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN \
    corepack enable && \
    corepack prepare pnpm@9.15.0 --activate && \
    apk --no-cache add dumb-init=1.2.5-r3

COPY apps/webapp/package.json apps/webapp/pnpm-lock.yaml ./

RUN pnpm install --frozen-lockfile

FROM webapp-deps AS webapp-build

COPY apps/webapp/ ./

RUN pnpm build

FROM nginx:1.28.0-alpine3.21-slim AS webapp

COPY --from=webapp-build /usr/local/src/oxid/dist/client /usr/local/src/oxid
COPY apps/webapp/nginx.conf /etc/nginx/conf.d/default.conf
COPY apps/webapp/docker-entrypoint.sh /docker-entrypoint.d/docker-entrypoint.sh

EXPOSE 80

RUN chmod +x /docker-entrypoint.d/docker-entrypoint.sh
