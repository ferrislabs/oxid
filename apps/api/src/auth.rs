use auth::{Identity, Token};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use http::{HeaderValue, header::AUTHORIZATION};
use tracing::error;

use crate::{
    errors::{ApiError, MiddlewareError},
    state::AppState,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, MiddlewareError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or(MiddlewareError::MissingAuthHeader)?;

    let token = extract_token_from_bearer(auth_header)
        .await
        .map_err(|_| MiddlewareError::InvalidAuthHeader)?;

    let identity = state
        .service
        .auth
        .get_identity(token.as_str())
        .await
        .map_err(|e| {
            error!("Auth middleware: failed to identify user {:?}", e);
            MiddlewareError::AuthenticationFailed(e.to_string())
        })?;

    if let Identity::User(user) = &identity {
        let name = user.name.clone().unwrap_or_else(|| user.username.clone());
        let email = user
            .email
            .clone()
            .unwrap_or_else(|| format!("{}@local", user.username));

        // let command = CreateUserCommand {
        //     name,
        //     email,
        //     sub: user.id.clone(),
        // };
        // if let Err(err) = state.create_user(command).await {
        //     error!("Auth middleware: failed to create user {:?}", err);
        // }
    }

    req.extensions_mut().insert(identity);

    Ok(next.run(req).await)
}

pub async fn extract_token_from_bearer(auth_header: &HeaderValue) -> Result<Token, ApiError> {
    let auth_str = auth_header.to_str().map_err(|_| ApiError::TokenNotFound)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(ApiError::TokenNotFound);
    }

    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or(ApiError::TokenNotFound)?;

    Ok(Token::new(token.to_string()))
}
