use crate::{AuthError, AuthRepository, Identity};

#[derive(Clone)]
pub struct AuthService<A>
where
    A: AuthRepository,
{
    auth_repository: A,
}

impl<A> AuthService<A>
where
    A: AuthRepository,
{
    pub fn new(auth_repository: A) -> Self {
        Self { auth_repository }
    }

    pub async fn get_identity(&self, token: &str) -> Result<Identity, AuthError> {
        self.auth_repository.identify(token).await
    }
}
