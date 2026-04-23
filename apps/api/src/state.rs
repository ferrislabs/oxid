use std::sync::Arc;

use args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
}
