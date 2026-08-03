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
async fn an_organization_is_currently_invisible_to_its_own_owner() {
    // Characterisation of the first-login loop — see issue #30.
    //
    // The handler passes the OIDC subject where the query expects the internal
    // user id, so the join never matches. Once the identifier spaces are
    // separated this must assert that the organization IS listed, and the test
    // renamed accordingly.
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
        0,
        "today the owner sees nothing; issue #30 must make this 1"
    );
}

// --- Cross-tenant exposure ------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn listing_organizations_currently_exposes_every_tenant() {
    // Characterisation of the cross-tenant leak — see issue #31.
    //
    // One database, two callers: Alice creates an organization, Bob is a
    // stranger to it. The listing endpoint applies no authorization and no
    // tenant predicate, so Bob sees it. Once scoped, this must assert that
    // Bob's listing is empty, and the test renamed accordingly.
    let api = TestApi::start().await;
    create_org(&api, &alice_token(&api), "Acme", "acme").await;

    let response = api
        .get("/api/v1/organizations")
        .bearer_auth(bob_token(&api))
        .send()
        .await
        .expect("request reaches the api");

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await.expect("json body");
    assert_eq!(
        body["data"].as_array().expect("data is an array").len(),
        1,
        "Bob is a member of nothing, yet sees Alice's organization"
    );
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
async fn updating_is_currently_denied_even_to_the_owner() {
    // Characterisation — see issue #30.
    //
    // Membership enrichment compares the OIDC subject against a column holding
    // internal user ids, so it denies everyone including the owner. Once the
    // identifier spaces are separated this must assert 200 for the owner, and
    // the test renamed accordingly.
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
        StatusCode::FORBIDDEN,
        "today even the owner is denied; issue #30 must make this 200"
    );
}

// --- Pagination -----------------------------------------------------------

#[tokio::test]
#[ignore = "requires docker"]
async fn an_out_of_range_page_currently_breaks_the_listing() {
    // Characterisation — see issue #37.
    //
    // The page parameter is unbounded and the offset is computed without
    // checked arithmetic, so a large page produces a negative OFFSET in release
    // and a panic in debug. Once bounded, this must assert a client error.
    let api = TestApi::start().await;

    let response = api
        .get("/api/v1/organizations?page=18446744073709551615")
        .bearer_auth(alice_token(&api))
        .send()
        .await;

    match response {
        // Debug profile: the arithmetic panics and the connection is dropped.
        Err(_) => {}
        // Release profile: the negative offset reaches Postgres.
        Ok(r) => assert_eq!(
            r.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "issue #37 must turn this into a client error"
        ),
    }
}
