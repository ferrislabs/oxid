extern crate self as oxid_core;

pub mod application;
pub mod infrastructure;
pub(crate) mod domain;

pub use application::*;
pub use domain::{User, UserId};
