use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{User, UserId, commands::CreateUserCommand, ports::UserRepository};

#[derive(Clone)]
pub struct UserService<U>
where
    U: UserRepository,
{
    user_repository: U,
}

impl<U> UserService<U>
where
    U: UserRepository,
{
    pub fn new(user_repository: U) -> Self {
        Self { user_repository }
    }

    pub async fn create_user(&self, command: CreateUserCommand) -> Result<User, CoreError> {
        let now = Utc::now();
        let user = User {
            id: UserId(generate_uuid_v7()),
            name: command.name,
            email: command.email,
            sub: command.sub,
            created_at: now,
            updated_at: now,
        };

        let user = self.user_repository.upsert_by_email(&user).await?;

        Ok(user)
    }
}
