pub mod auth;
pub mod talks;
pub mod labels;
pub mod ratings;
pub mod dashboard;
pub mod tracks;
pub mod schedule_slots;

pub use auth::{login, register};
pub use talks::{create_talk, delete_talk, get_my_talks, get_talk, update_talk, upload_slides, respond_to_talk, list_all_talks, change_talk_state};
pub use labels::{list_labels, create_label, update_label, delete_label, get_talk_labels, add_labels_to_talk, remove_label_from_talk};
pub use ratings::{create_or_update_rating, get_talk_ratings, delete_rating, get_my_rating, get_ratings_statistics};
pub use dashboard::get_dashboard_stats;
pub use tracks::{list_tracks, get_track, create_track, update_track, delete_track};
pub use schedule_slots::{list_schedule_slots, get_schedule_slot, create_schedule_slot, update_schedule_slot, delete_schedule_slot, assign_talk_to_slot, unassign_talk_from_slot};
