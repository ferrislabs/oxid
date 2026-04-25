use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Wraps an async method body in a SQL transaction.
///
/// Conventions assumed by the macro:
/// - the method takes `&self` and `self` has a `pool: PgPool` field;
/// - the method body uses a binding named `tx` for the transaction;
/// - `oxid_core::infrastructure::postgres::with_tx` is reachable.
///
/// # Example
/// ```ignore
/// #[transactional]
/// pub async fn create_user(&self, cmd: CreateUserCommand) -> Result<User, CoreError> {
///     let mut service = UserService::new(PgUserRepository::new(tx));
///     service.create_user(cmd).await
/// }
/// ```
#[proc_macro_attribute]
pub fn transactional(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            ::oxid_core::infrastructure::postgres::with_tx(&self.pool, async |tx| #block).await
        }
    };

    expanded.into()
}
