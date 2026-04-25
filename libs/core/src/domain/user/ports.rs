use common::CoreError;

use crate::User;

pub trait UserRepository: Send {
    fn upsert_by_email(
        &mut self,
        user: &User,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;
}
