pub mod auth;
pub mod talks;
pub mod labels;
pub mod ratings;

pub use auth::{login, register};
pub use talks::{create_talk, delete_talk, get_my_talks, get_talk, update_talk, upload_slides, respond_to_talk, list_all_talks};
pub use labels::{list_labels, create_label, update_label, delete_label, get_talk_labels, add_labels_to_talk, remove_label_from_talk};
pub use ratings::{create_or_update_rating, get_talk_ratings, delete_rating, get_my_rating};
