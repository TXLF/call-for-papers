pub mod auth;
pub mod conference;
pub mod email_template;
pub mod label;
pub mod rating;
pub mod schedule_slot;
pub mod talk;
pub mod track;
pub mod user;

pub use auth::{AuthResponse, Claims, LoginRequest, RegisterRequest};
pub use conference::{
    Conference, ConferenceResponse, CreateConferenceRequest, UpdateConferenceRequest,
};
pub use email_template::{
    CreateEmailTemplateRequest, EmailTemplate, EmailTemplateResponse, UpdateEmailTemplateRequest,
};
pub use label::{
    AddLabelToTalkRequest, CreateLabelRequest, Label, LabelResponse, TalkLabel, UpdateLabelRequest,
};
pub use rating::{
    CreateRatingRequest, Rating, RatingDistribution, RatingResponse, RatingsStatisticsResponse,
    TalkRatingStats, UpdateRatingRequest,
};
pub use schedule_slot::{
    AssignTalkRequest, CreateScheduleSlotRequest, PublicScheduleSlot, PublicScheduleTalk,
    ScheduleSlot, ScheduleSlotResponse, UpdateScheduleSlotRequest,
};
pub use talk::{
    ChangeStateRequest, CreateTalkRequest, RespondToTalkRequest, Talk, TalkAction, TalkResponse,
    TalkState, UpdateTalkRequest,
};
pub use track::{CreateTrackRequest, Track, TrackResponse, UpdateTrackRequest};
pub use user::User;
