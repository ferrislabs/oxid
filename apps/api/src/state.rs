use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
}
