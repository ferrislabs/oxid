//! Test harness driving the real organization router over HTTP.
//!
//! Each [`TestApi`] owns a throwaway Postgres container with the migrations
//! applied, so the queries under test run against the real schema — tenant
//! isolation and uniqueness live in constraints, not in Rust, and mocking the
//! driver would not exercise them.
//!
//! Authentication is injected through the existing `AuthRepository` port via a
//! fixed-identity adapter, so no test reaches a live identity provider. Rate
//! limiting uses an always-allow adapter: its real behaviour is covered against
//! Redis in the `rate-limit` crate, and starting a second container per test
//! would only slow the suite.

#![allow(dead_code)] // helpers land ahead of the sub-issues that consume them

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use auth::{AuthService, FixedIdentityRepository, Identity, User};
use axum::Router;
use clap::Parser;
use handlers::AppState;
use oxid_core::{OxidAuthRepository, OxidRateLimiter, OxidUseCase, default_authorizer};
use rate_limit::{AlwaysAllowLimiter, Quota, RateLimitService};
use reqwest::{Client, RequestBuilder};
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

pub struct TestApi {
    /// Held so the container outlives the test; dropping it stops Postgres.
    _postgres: ContainerAsync<Postgres>,
    base_url: String,
    client: Client,
    pub pool: PgPool,
}

impl TestApi {
    /// Starts the API with every token rejected — the shape to use when the
    /// test is about unauthenticated access.
    pub async fn start() -> Self {
        Self::boot(FixedIdentityRepository::rejecting()).await
    }

    /// Starts the API with every token resolving to `identity`.
    pub async fn start_authenticated_as(identity: Identity) -> Self {
        Self::boot(FixedIdentityRepository::authenticating_as(identity)).await
    }

    /// Starts the API where each token resolves to its own identity, so a
    /// single instance — and so a single database — serves several callers.
    /// This is the shape required to exercise cross-tenant access.
    pub async fn start_with_identities<I, T>(tokens: I) -> Self
    where
        I: IntoIterator<Item = (T, Identity)>,
        T: Into<String>,
    {
        Self::boot(FixedIdentityRepository::with_tokens(tokens)).await
    }

    async fn boot(auth_repo: FixedIdentityRepository) -> Self {
        // Same image as docker-compose: the migrations call `gen_random_uuid()`,
        // which is only built in from PostgreSQL 13 onwards and is declared
        // nowhere. Testing on an older image would fail for a reason unrelated
        // to the code under test.
        let postgres = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .expect("start postgres container");
        let port = postgres
            .get_host_port_ipv4(5432)
            .await
            .expect("mapped postgres port");
        let host = postgres.get_host().await.expect("postgres host");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!("postgres://postgres:postgres@{host}:{port}/postgres"))
            .await
            .expect("connect to postgres");

        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let state = AppState {
            // Every argument currently has a default, so the binary parses with
            // no argv. Once secrets become required this must supply them.
            args: Arc::new(Args::parse_from(["oxid"])),
            auth: AuthService::new(OxidAuthRepository::Fixed(auth_repo)),
            usecase: OxidUseCase::new(pool.clone(), default_authorizer()),
            rate_limit: RateLimitService::new(OxidRateLimiter::AlwaysAllow(
                AlwaysAllowLimiter,
            )),
            rate_limit_quota: Quota::per_minute(1_000),
        };

        let router = Router::new()
            .merge(handlers_organization::router(&state))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");

        tokio::spawn(async move {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .expect("serve test api");
        });

        Self {
            _postgres: postgres,
            base_url: format!("http://{addr}"),
            client: Client::new(),
            pool,
        }
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.client.get(format!("{}{path}", self.base_url))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.client.post(format!("{}{path}", self.base_url))
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        self.client.patch(format!("{}{path}", self.base_url))
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.client.delete(format!("{}{path}", self.base_url))
    }
}

/// A user identity as the OIDC layer would produce it. `subject` is the OIDC
/// subject — which the handler layer currently parses as an internal user id,
/// so tests must pass a UUID here until that is separated.
pub fn user_identity(subject: &str, username: &str, email: &str) -> Identity {
    Identity::User(User {
        id: subject.to_owned(),
        email: Some(email.to_owned()),
        name: Some(username.to_owned()),
        roles: Vec::new(),
        username: username.to_owned(),
    })
}
