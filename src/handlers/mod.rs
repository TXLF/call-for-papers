pub mod ai_tagging;
pub mod auth;
pub mod bulk_email;
pub mod conferences;
pub mod config;
pub mod dashboard;
pub mod email_templates;
pub mod export;
pub mod labels;
pub mod ratings;
pub mod schedule_slots;
pub mod talks;
pub mod tracks;

pub use ai_tagging::{auto_tag_with_claude, create_ai_labels};
pub use auth::{
    apple_authorize, apple_callback, facebook_authorize, facebook_callback, github_authorize,
    github_callback, google_authorize, google_callback, linkedin_authorize, linkedin_callback,
    login, register,
};
pub use bulk_email::send_bulk_email;
pub use conferences::{
    create_conference, delete_conference, get_active_conference, get_conference, list_conferences,
    update_conference,
};
pub use config::get_configuration;
pub use dashboard::get_dashboard_stats;
pub use email_templates::{
    create_email_template, delete_email_template, get_email_template, list_email_templates,
    update_email_template,
};
pub use export::export_talks;
pub use labels::{
    add_labels_to_talk, create_label, delete_label, get_talk_labels, list_labels,
    remove_label_from_talk, update_label,
};
pub use ratings::{
    create_or_update_rating, delete_rating, get_my_rating, get_ratings_statistics, get_talk_ratings,
};
pub use schedule_slots::{
    assign_talk_to_slot, create_schedule_slot, delete_schedule_slot, get_public_schedule,
    get_schedule_slot, list_schedule_slots, unassign_talk_from_slot, update_schedule_slot,
};
pub use talks::{
    change_talk_state, create_talk, delete_talk, get_my_talks, get_talk, list_all_talks,
    respond_to_talk, update_talk, upload_slides,
};
pub use tracks::{create_track, delete_track, get_track, list_tracks, update_track};
