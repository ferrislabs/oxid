//! HTTP integration tests for the organization router.
//!
//! Run with: `cargo test -p handlers-organization -- --ignored` (requires
//! Docker, and `SQLX_OFFLINE=true` unless a database is reachable at build
//! time). On a Docker runtime that does not expose `/var/run/docker.sock`
//! — OrbStack, Colima — export `DOCKER_HOST` first.
//!
//! Some tests are named `..._currently_...` and assert behaviour the audit
//! identified as defective. They are characterisation tests: they pin what the
//! system does today so the sub-issue fixing it has to flip the assertion, and
//! so the suite stays green in the meantime. Each names the issue that owns it.

mod harness;

use harness::TestApi;
use http::StatusCode;
use serde_json::json;

const ALICE: &str = "11111111-1111-4111-8111-111111111111";
const BOB: &str = "22222222-2222-4222-8222-222222222222";

fn alice_token(api: &TestApi) -> String {
    api.token(ALICE, "alice", "alice@example.com")
}

fn bob_token(api: &TestApi) -> String {
    api.token(BOB, "bob", "bob@example.com")
}

/// Creates an organization over HTTP, as the caller behind `token`, and
/// returns its id.
async fn create_org(api: &TestApi, token: &str, name: &str, slug: &str) -> String {
    let response = api
        .post("/api/v1/organizations")
        .bearer_auth(token)
        .json(&json!({ "name": name, "slug": slug }))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "organization creation should succeed"
    );

    let body: serde_json::Value = response.json().await.expect("json body");
    body["data"]["id"].as_str().expect("id in payload").to_owned()
}

// --- Authentication -------------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn listing_organizations_without_a_token_is_rejected() {
    let api = TestApi::start().await;

    let response = api
        .get("/api/v1/organizations")
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires docker"]
async fn a_token_the_provider_rejects_is_refused() {
    let api = TestApi::start().await;

    let response = api
        .get("/api/v1/users/@me/organizations")
        .bearer_auth("not-a-valid-token")
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// --- Creation -------------------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn an_authenticated_user_can_create_an_organization() {
    let api = TestApi::start().await;

    let response = api
        .post("/api/v1/organizations")
        .bearer_auth(alice_token(&api))
        .json(&json!({ "name": "Acme", "slug": "acme" }))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: serde_json::Value = response.json().await.expect("json body");
    assert_eq!(body["data"]["slug"], "acme");
}

#[tokio::test]
#[ignore = "requires docker"]
async fn reusing_a_slug_is_a_conflict() {
    let api = TestApi::start().await;
    create_org(&api, &alice_token(&api), "Acme", "acme").await;

    let response = api
        .post("/api/v1/organizations")
        .bearer_auth(alice_token(&api))
        .json(&json!({ "name": "Acme Again", "slug": "acme" }))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// --- The caller's own organizations ---------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn an_organization_is_listed_for_its_owner() {
    let api = TestApi::start().await;
    create_org(&api, &alice_token(&api), "Acme", "acme").await;

    let response = api
        .get("/api/v1/users/@me/organizations")
        .bearer_auth(alice_token(&api))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await.expect("json body");
    assert_eq!(
        body["data"].as_array().expect("data is an array").len(),
        1,
        "the organization its owner just created must be listed for them"
    );
}

// --- Cross-tenant exposure ------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn the_global_organization_listing_is_no_longer_exposed() {
    // The endpoint returned every tenant's organizations with no authorization
    // check at all. It had no working authorization model and no consumer, and
    // its caller-scoped equivalent already exists, so it was removed rather
    // than guarded. Creation still lives on this path, hence 405 and not 404.
    let api = TestApi::start().await;
    create_org(&api, &alice_token(&api), "Acme", "acme").await;

    let response = api
        .get("/api/v1/organizations")
        .bearer_auth(bob_token(&api))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// --- Update ---------------------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn updating_an_organization_as_a_non_member_is_denied() {
    // One database, two callers: Bob genuinely exists and is genuinely not a
    // member of Alice's organization.
    let api = TestApi::start().await;
    let org_id = create_org(&api, &alice_token(&api), "Acme", "acme").await;

    let response = api
        .patch(&format!("/api/v1/organizations/{org_id}"))
        .bearer_auth(bob_token(&api))
        .json(&json!({ "name": "Hijacked", "slug": "hijacked" }))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "a non-member must not be able to update another tenant's organization"
    );
}

#[tokio::test]
#[ignore = "requires docker"]
async fn an_owner_can_update_their_organization() {
    let api = TestApi::start().await;
    let org_id = create_org(&api, &alice_token(&api), "Acme", "acme").await;

    let response = api
        .patch(&format!("/api/v1/organizations/{org_id}"))
        .bearer_auth(alice_token(&api))
        .json(&json!({ "name": "Acme Renamed", "slug": "acme-renamed" }))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "the owner holds every permission on their own organization"
    );
}

// --- Schema invariants ----------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn a_role_cannot_be_assigned_to_a_member_of_another_organization() {
    // `member_roles` links a member to a role with no organization of its own,
    // so nothing in the schema forbids pairing a member of one organization
    // with a role belonging to another. The application must not be the only
    // thing standing between a caller and a cross-tenant privilege grant.
    let api = TestApi::start().await;
    let acme = create_org(&api, &alice_token(&api), "Acme", "acme").await;
    let other = create_org(&api, &bob_token(&api), "Other", "other").await;

    let member_of_acme: uuid::Uuid = sqlx::query_scalar(
        "SELECT id FROM organization_members WHERE organization_id = $1 LIMIT 1",
    )
    .bind(uuid::Uuid::parse_str(&acme).expect("acme id"))
    .fetch_one(&api.pool)
    .await
    .expect("acme has a member");

    let role_of_other: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM roles WHERE organization_id = $1 LIMIT 1")
            .bind(uuid::Uuid::parse_str(&other).expect("other id"))
            .fetch_one(&api.pool)
            .await
            .expect("other has roles");

    let result = sqlx::query(
        "INSERT INTO member_roles (id, member_id, role_id) VALUES (gen_random_uuid(), $1, $2)",
    )
    .bind(member_of_acme)
    .bind(role_of_other)
    .execute(&api.pool)
    .await;

    assert!(
        result.is_err(),
        "the database must reject a member paired with another organization's role"
    );
}

// --- Identity provisioning ------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn two_identities_without_an_email_are_two_distinct_users() {
    // A realm that does not release the email claim must still yield one row
    // per subject. Substituting a constant for the missing claim collapses
    // every such identity onto a single row, and with it their memberships.
    let api = TestApi::start().await;

    for (subject, username) in [(ALICE, "alice"), (BOB, "bob")] {
        let response = api
            .get("/api/v1/users/@me/organizations")
            .bearer_auth(api.token_without_email(subject, username))
            .send()
            .await
            .expect("request reaches the api");
        assert_eq!(response.status(), StatusCode::OK, "{username} should authenticate");
    }

    let subjects: Vec<String> = sqlx::query_scalar("SELECT sub FROM users ORDER BY sub")
        .fetch_all(&api.pool)
        .await
        .expect("read users");

    assert_eq!(subjects, vec![ALICE.to_owned(), BOB.to_owned()]);
}

// --- Soft delete ----------------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn a_slug_is_released_when_its_organization_is_deleted() {
    // The uniqueness constraint spanned the whole table, deleted rows included,
    // so a slug stayed reserved forever. Recreating an organization the caller
    // had deleted failed with "slug already taken" — about an organization no
    // read path can show them.
    let api = TestApi::start().await;
    let token = alice_token(&api);
    let org_id = create_org(&api, &token, "Acme", "acme").await;

    sqlx::query("UPDATE organizations SET deleted_at = now() WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&org_id).expect("org id"))
        .execute(&api.pool)
        .await
        .expect("soft delete");

    let response = api
        .post("/api/v1/organizations")
        .bearer_auth(&token)
        .json(&serde_json::json!({ "name": "Acme Again", "slug": "acme" }))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "the slug of a deleted organization must be available again"
    );
}

#[tokio::test]
#[ignore = "requires docker"]
async fn a_deleted_organization_cannot_be_updated_by_its_owner() {
    // Membership lookups did not join `organizations`, so a deleted tenant
    // still had members and every organization-scoped path stayed reachable.
    let api = TestApi::start().await;
    let token = alice_token(&api);
    let org_id = create_org(&api, &token, "Acme", "acme").await;

    sqlx::query("UPDATE organizations SET deleted_at = now() WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&org_id).expect("org id"))
        .execute(&api.pool)
        .await
        .expect("soft delete");

    let response = api
        .patch(&format!("/api/v1/organizations/{org_id}"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "name": "Revived", "slug": "revived" }))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// --- Authentication cost --------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn the_key_set_is_not_refetched_on_every_request() {
    // Validation fetched the realm's key set on every single call, so each
    // request - including one carrying a bogus token - caused an outbound call
    // to the identity provider.
    let api = TestApi::start().await;
    let token = alice_token(&api);

    for _ in 0..5 {
        api.get("/api/v1/users/@me/organizations")
            .bearer_auth(&token)
            .send()
            .await
            .expect("request reaches the api");
    }

    assert!(
        api.jwks_fetches() < 5,
        "expected the key set to be cached, it was fetched {} times for 5 requests",
        api.jwks_fetches()
    );
}

// --- Routing hygiene ------------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn unimplemented_routes_are_not_mounted() {
    // Both were routed to `todo!()`, which panics: calling them aborted the
    // connection and the OpenAPI document advertised a success response.
    // A route that does not exist is a better answer than one that panics.
    let api = TestApi::start().await;
    let token = alice_token(&api);
    let org_id = create_org(&api, &token, "Acme", "acme").await;

    for method in ["GET", "DELETE"] {
        let request = match method {
            "GET" => api.get(&format!("/api/v1/organizations/{org_id}")),
            _ => api.delete(&format!("/api/v1/organizations/{org_id}")),
        };
        let response = request
            .bearer_auth(&token)
            .send()
            .await
            .unwrap_or_else(|e| panic!("{method} must answer, not drop the connection: {e}"));

        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} on an organization should not be routed"
        );
    }
}
