use common::CoreError;
use oxid_macros::transactional;
use sqlx::PgPool;

use crate::{
    User,
    domain::user::{commands::CreateUserCommand, service::UserService},
    infrastructure::user::postgres::PgUserRepository,
};

#[derive(Clone)]
pub struct UserUseCase {
    pool: PgPool,
}

impl UserUseCase {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[transactional]
    pub async fn create_user(&self, command: CreateUserCommand) -> Result<User, CoreError> {
        let mut service = UserService::new(PgUserRepository::new(tx));
        service.create_user(command).await
    }
}
