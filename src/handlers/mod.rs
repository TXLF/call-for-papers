pub mod auth;
pub mod talks;

pub use auth::{register, login};
pub use talks::{create_talk, get_my_talks, get_talk, update_talk, delete_talk};
