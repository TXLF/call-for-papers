use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    app::Route,
    components::LabelBadge,
    services::talks::TalkService,
    types::{Talk, TalkState},
};

#[function_component(MyTalks)]
pub fn my_talks() -> Html {
    let navigator = use_navigator().unwrap();
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let responding_to = use_state(|| None::<String>);
    let success_message = use_state(|| None::<String>);

    {
        let talks = talks.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match TalkService::get_my_talks().await {
                    Ok(fetched_talks) => {
                        talks.set(fetched_talks);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_submit_new = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::SubmitTalk);
        })
    };

    let create_respond_handler = |talk_id: String, action: String| {
        let talks = talks.clone();
        let responding_to = responding_to.clone();
        let error = error.clone();
        let success_message = success_message.clone();

        Callback::from(move |_: MouseEvent| {
            let talk_id = talk_id.clone();
            let action = action.clone();
            let talks = talks.clone();
            let responding_to = responding_to.clone();
            let error = error.clone();
            let success_message = success_message.clone();

            // Clear previous messages
            error.set(None);
            success_message.set(None);
            responding_to.set(Some(talk_id.clone()));

            wasm_bindgen_futures::spawn_local(async move {
                match TalkService::respond_to_talk(&talk_id, &action).await {
                    Ok(updated_talk) => {
                        // Update the talk in the list
                        let mut current_talks = (*talks).clone();
                        if let Some(talk) = current_talks.iter_mut().find(|t| t.id == talk_id) {
                            *talk = updated_talk;
                        }
                        talks.set(current_talks);

                        let message = if action == "accept" {
                            "Talk accepted successfully!"
                        } else {
                            "Talk declined"
                        };
                        success_message.set(Some(message.to_string()));
                        responding_to.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to respond: {}", e)));
                        responding_to.set(None);
                    }
                }
            });
        })
    };

    html! {
        <div class="talks-container">
            <div class="talks-header">
                <h2>{ "My Talk Submissions" }</h2>
                <button onclick={on_submit_new.clone()} class="btn-primary">
                    { "Submit New Talk" }
                </button>
            </div>

            {
                if let Some(msg) = (*success_message).as_ref() {
                    html! {
                        <div class="success-message">
                            { msg }
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if *loading {
                    html! {
                        <div class="loading">{ "Loading your talks..." }</div>
                    }
                } else if let Some(err) = (*error).as_ref() {
                    html! {
                        <div class="error-message">
                            { err }
                        </div>
                    }
                } else if talks.is_empty() {
                    html! {
                        <div class="empty-state">
                            <p>{ "You haven't submitted any talks yet." }</p>
                            <button onclick={on_submit_new.clone()} class="btn-primary">
                                { "Submit Your First Talk" }
                            </button>
                        </div>
                    }
                } else {
                    html! {
                        <div class="talks-list">
                            {
                                for talks.iter().map(|talk| {
                                    let state_class = format!("state-{:?}", talk.state).to_lowercase();
                                    let is_pending = matches!(talk.state, TalkState::Pending);
                                    let is_responding = (*responding_to).as_ref() == Some(&talk.id);

                                    // State-specific display information
                                    let (state_icon, state_label, state_help) = match talk.state {
                                        TalkState::Submitted => (
                                            "📝",
                                            "Submitted",
                                            "Your submission has been received. The organizing team is reviewing all proposals."
                                        ),
                                        TalkState::Pending => (
                                            "⏳",
                                            "Awaiting Response",
                                            "Congratulations! Your talk has been selected. Please confirm your attendance or decline below."
                                        ),
                                        TalkState::Accepted => (
                                            "✅",
                                            "Accepted",
                                            "Your talk has been confirmed! You'll receive more details about the conference schedule soon."
                                        ),
                                        TalkState::Rejected => (
                                            "❌",
                                            "Not Selected",
                                            "Thank you for your submission. While this talk wasn't selected, we encourage you to submit again in the future."
                                        ),
                                    };

                                    html! {
                                        <div class={classes!("talk-card", state_class.clone())} key={talk.id.clone()}>
                                            <div class="talk-header">
                                                <h3>{ &talk.title }</h3>
                                                <span class={classes!("talk-status", state_class)}>
                                                    <span class="status-icon">{ state_icon }</span>
                                                    <span class="status-label">{ state_label }</span>
                                                </span>
                                            </div>

                                            <div class="talk-status-help">
                                                { state_help }
                                            </div>
                                            <p class="talk-summary">{ &talk.short_summary }</p>
                                            {
                                                if !talk.labels.is_empty() {
                                                    html! {
                                                        <div class="talk-labels">
                                                            {
                                                                for talk.labels.iter().map(|label| {
                                                                    html! {
                                                                        <LabelBadge
                                                                            label={label.clone()}
                                                                            removable={false}
                                                                        />
                                                                    }
                                                                })
                                                            }
                                                        </div>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(desc) = &talk.long_description {
                                                    html! {
                                                        <details class="talk-description">
                                                            <summary>{ "View full description" }</summary>
                                                            <p>{ desc }</p>
                                                        </details>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(slides_url) = &talk.slides_url {
                                                    html! {
                                                        <div class="talk-slides">
                                                            <a href={slides_url.clone()} target="_blank" class="slides-link">
                                                                { "📄 View Slides" }
                                                            </a>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }

                                            // Action buttons for pending talks
                                            {
                                                if is_pending {
                                                    html! {
                                                        <div class="talk-actions">
                                                            <p class="pending-notice">
                                                                { "Your talk has been selected! Please respond:" }
                                                            </p>
                                                            <div class="action-buttons">
                                                                <button
                                                                    onclick={create_respond_handler(talk.id.clone(), "accept".to_string())}
                                                                    disabled={is_responding}
                                                                    class="btn-accept"
                                                                >
                                                                    { if is_responding { "Processing..." } else { "Accept" } }
                                                                </button>
                                                                <button
                                                                    onclick={create_respond_handler(talk.id.clone(), "decline".to_string())}
                                                                    disabled={is_responding}
                                                                    class="btn-decline"
                                                                >
                                                                    { if is_responding { "Processing..." } else { "Decline" } }
                                                                </button>
                                                            </div>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }

                                            <div class="talk-meta">
                                                <div class="meta-item">
                                                    <span class="meta-label">{ "Submitted:" }</span>
                                                    <span class="meta-value">{ &talk.submitted_at[..10] }</span>
                                                </div>
                                                {
                                                    if talk.updated_at != talk.submitted_at {
                                                        html! {
                                                            <div class="meta-item">
                                                                <span class="meta-label">{ "Last Updated:" }</span>
                                                                <span class="meta-value">{ &talk.updated_at[..10] }</span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>
                                        </div>
                                    }
                                })
                            }
                        </div>
                    }
                }
            }
        </div>
    }
}
