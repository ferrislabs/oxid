use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{User, UserId, domain::user::commands::CreateUserCommand, domain::user::ports::UserRepository};

pub struct UserService<U>
where
    U: UserRepository,
{
    repo: U,
}

impl<U> UserService<U>
where
    U: UserRepository,
{
    pub fn new(repo: U) -> Self {
        Self { repo }
    }

    pub async fn create_user(&mut self, command: CreateUserCommand) -> Result<User, CoreError> {
        let now = Utc::now();
        let user = User {
            id: UserId(generate_uuid_v7()),
            name: command.name,
            username: command.username,
            email: command.email,
            sub: command.sub,
            created_at: now,
            updated_at: now,
        };

        self.repo.upsert_by_email(&user).await
    }
}
