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
