use yew::prelude::*;
use yew_router::prelude::*;
use crate::{
    app::Route,
    services::talks::TalkService,
    types::{Talk, TalkState},
};

#[function_component(SpeakerDashboard)]
pub fn speaker_dashboard() -> Html {
    let navigator = use_navigator().unwrap();
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let responding_to = use_state(|| None::<String>);

    // Fetch talks on mount
    {
        let talks = talks.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match TalkService::get_my_talks().await {
                    Ok(data) => {
                        talks.set(data);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Calculate stats
    let total_talks = talks.len();
    let submitted_count = talks.iter().filter(|t| matches!(t.state, TalkState::Submitted)).count();
    let pending_count = talks.iter().filter(|t| matches!(t.state, TalkState::Pending)).count();
    let accepted_count = talks.iter().filter(|t| matches!(t.state, TalkState::Accepted)).count();
    let rejected_count = talks.iter().filter(|t| matches!(t.state, TalkState::Rejected)).count();

    // Get pending talks for action items
    let pending_talks: Vec<Talk> = talks.iter()
        .filter(|t| matches!(t.state, TalkState::Pending))
        .cloned()
        .collect();

    // Get recent talks (last 5)
    let mut recent_talks: Vec<Talk> = (*talks).clone();
    recent_talks.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
    recent_talks.truncate(5);

    // Response handler for pending talks
    let create_respond_handler = |talk_id: String, action: String| {
        let talks = talks.clone();
        let responding_to = responding_to.clone();
        let error = error.clone();

        Callback::from(move |_: MouseEvent| {
            let talk_id = talk_id.clone();
            let action = action.clone();
            let talks = talks.clone();
            let responding_to = responding_to.clone();
            let error = error.clone();

            error.set(None);
            responding_to.set(Some(talk_id.clone()));

            wasm_bindgen_futures::spawn_local(async move {
                match TalkService::respond_to_talk(&talk_id, &action).await {
                    Ok(updated_talk) => {
                        let mut current_talks = (*talks).clone();
                        if let Some(talk) = current_talks.iter_mut().find(|t| t.id == talk_id) {
                            *talk = updated_talk;
                        }
                        talks.set(current_talks);
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
        <div class="speaker-dashboard">
            <h1>{ "My Speaker Dashboard" }</h1>

            if *loading {
                <div class="loading">{ "Loading your dashboard..." }</div>
            } else {
                <>
                    if let Some(err) = (*error).as_ref() {
                        <div class="error-message">{ err }</div>
                    }

                    // Overview Stats
                    <section class="dashboard-section">
                        <h2>{ "Overview" }</h2>
                        <div class="stats-grid">
                            <div class="stat-card primary">
                                <div class="stat-value">{ total_talks }</div>
                                <div class="stat-label">{ "Total Talks" }</div>
                            </div>

                            <div class="stat-card state-submitted">
                                <div class="stat-value">{ submitted_count }</div>
                                <div class="stat-label">{ "Under Review" }</div>
                            </div>

                            <div class="stat-card state-pending">
                                <div class="stat-value">{ pending_count }</div>
                                <div class="stat-label">{ "Awaiting Response" }</div>
                            </div>

                            <div class="stat-card state-accepted">
                                <div class="stat-value">{ accepted_count }</div>
                                <div class="stat-label">{ "Accepted" }</div>
                            </div>
                        </div>
                    </section>

                    // Pending Actions
                    if !pending_talks.is_empty() {
                        <section class="dashboard-section pending-actions-section">
                            <h2>{ "⏳ Action Required" }</h2>
                            <p class="section-description">
                                { "The following talks have been selected and need your response." }
                            </p>
                            <div class="pending-talks-list">
                                {
                                    pending_talks.iter().map(|talk| {
                                        let talk_id = talk.id.clone();
                                        let is_responding = (*responding_to).as_ref() == Some(&talk.id);

                                        html! {
                                            <div class="pending-talk-card" key={talk_id.clone()}>
                                                <div class="pending-talk-header">
                                                    <h3>{ &talk.title }</h3>
                                                    <span class="talk-status state-pending">
                                                        <span class="status-icon">{ "⏳" }</span>
                                                        <span class="status-label">{ "Awaiting Response" }</span>
                                                    </span>
                                                </div>
                                                <p class="pending-talk-summary">{ &talk.short_summary }</p>
                                                <div class="pending-talk-actions">
                                                    <button
                                                        onclick={create_respond_handler(talk_id.clone(), "accept".to_string())}
                                                        disabled={is_responding}
                                                        class="btn btn-accept"
                                                    >
                                                        { if is_responding { "Processing..." } else { "✅ Accept" } }
                                                    </button>
                                                    <button
                                                        onclick={create_respond_handler(talk_id, "decline".to_string())}
                                                        disabled={is_responding}
                                                        class="btn btn-decline"
                                                    >
                                                        { if is_responding { "Processing..." } else { "❌ Decline" } }
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        </section>
                    }

                    // Quick Actions
                    <section class="dashboard-section">
                        <h2>{ "Quick Actions" }</h2>
                        <div class="quick-actions">
                            <Link<Route> to={Route::SubmitTalk} classes="action-button primary">
                                <span class="action-icon">{ "➕" }</span>
                                <span class="action-text">{ "Submit New Talk" }</span>
                            </Link<Route>>

                            <Link<Route> to={Route::MyTalks} classes="action-button secondary">
                                <span class="action-icon">{ "📋" }</span>
                                <span class="action-text">{ "View All Submissions" }</span>
                            </Link<Route>>
                        </div>
                    </section>

                    // Recent Talks
                    {
                        if !recent_talks.is_empty() {
                            html! {
                                <section class="dashboard-section">
                                    <h2>{ "Recent Submissions" }</h2>
                                    <div class="recent-talks">
                                        {
                                            recent_talks.iter().map(|talk| {
                                                let (state_icon, state_label) = match talk.state {
                                                    TalkState::Submitted => ("📝", "Submitted"),
                                                    TalkState::Pending => ("⏳", "Awaiting Response"),
                                                    TalkState::Accepted => ("✅", "Accepted"),
                                                    TalkState::Rejected => ("❌", "Not Selected"),
                                                };

                                                let state_class = format!("state-{:?}", talk.state).to_lowercase();

                                                html! {
                                                    <div class="recent-talk-card" key={talk.id.clone()}>
                                                        <div class="recent-talk-header">
                                                            <h3>{ &talk.title }</h3>
                                                            <span class={classes!("talk-status", state_class)}>
                                                                <span class="status-icon">{ state_icon }</span>
                                                                <span class="status-label">{ state_label }</span>
                                                            </span>
                                                        </div>
                                                        <p class="recent-talk-summary">{ &talk.short_summary }</p>
                                                        <div class="recent-talk-meta">
                                                            <span>{ format!("Submitted: {}", &talk.submitted_at[..10]) }</span>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Html>()
                                        }
                                    </div>
                                    <Link<Route> to={Route::MyTalks} classes="btn btn-secondary view-all-btn">
                                        { "View All Submissions →" }
                                    </Link<Route>>
                                </section>
                            }
                        } else {
                            html! {
                                <section class="dashboard-section empty-state">
                                    <div class="empty-icon">{ "📝" }</div>
                                    <h2>{ "No Submissions Yet" }</h2>
                                    <p>{ "Get started by submitting your first talk proposal!" }</p>
                                    <Link<Route> to={Route::SubmitTalk} classes="btn btn-primary">
                                        { "Submit Your First Talk" }
                                    </Link<Route>>
                                </section>
                            }
                        }
                    }
                </>
            }
        </div>
    }
}
