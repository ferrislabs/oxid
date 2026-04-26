use common::CoreError;
use oxid_macros::transactional;

use crate::{
    User,
    application::OxidUseCase,
    domain::user::{commands::CreateUserCommand, service::UserService},
    infrastructure::user::postgres::PgUserRepository,
};

impl OxidUseCase {
    #[transactional]
    pub async fn create_user(&self, command: CreateUserCommand) -> Result<User, CoreError> {
        let mut service = UserService::new(PgUserRepository::new(tx));
        service.create_user(command).await
    }
}
