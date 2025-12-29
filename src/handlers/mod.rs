pub mod auth;
pub mod talks;

pub use auth::{login, register};
pub use talks::{create_talk, delete_talk, get_my_talks, get_talk, update_talk, upload_slides, respond_to_talk};
