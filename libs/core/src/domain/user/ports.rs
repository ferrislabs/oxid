use common::CoreError;

use crate::User;

pub trait UserRepository: Send + Sync {
    fn upsert_by_email(&self, user: &User) -> impl Future<Output = Result<User, CoreError>> + Send;
    fn find_by_sub(
        &self,
        sub: &str,
    ) -> impl Future<Output = Result<Option<User>, CoreError>> + Send;
}
