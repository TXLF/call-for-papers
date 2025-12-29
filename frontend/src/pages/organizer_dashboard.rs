use yew::prelude::*;
use yew_router::prelude::*;
use crate::{
    app::Route,
    components::RatingStars,
    services::dashboard::DashboardService,
    types::{DashboardStats, TalkState},
};

#[function_component(OrganizerDashboard)]
pub fn organizer_dashboard() -> Html {
    let stats = use_state(|| None::<DashboardStats>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Fetch dashboard stats on mount
    {
        let stats = stats.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match DashboardService::get_stats().await {
                    Ok(data) => {
                        stats.set(Some(data));
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

    if *loading {
        return html! {
            <div class="organizer-dashboard">
                <div class="loading">{ "Loading dashboard..." }</div>
            </div>
        };
    }

    if let Some(err) = (*error).as_ref() {
        return html! {
            <div class="organizer-dashboard">
                <div class="error">{ format!("Error: {}", err) }</div>
            </div>
        };
    }

    if let Some(data) = (*stats).as_ref() {
        html! {
            <div class="organizer-dashboard">
                <h1>{ "Organizer Dashboard" }</h1>

                // Overview Stats
                <section class="dashboard-section">
                    <h2>{ "Overview" }</h2>
                    <div class="stats-grid">
                        <div class="stat-card primary">
                            <div class="stat-value">{ data.total_talks }</div>
                            <div class="stat-label">{ "Total Talks" }</div>
                        </div>

                        <div class="stat-card info">
                            <div class="stat-value">{ data.talks_by_state.submitted }</div>
                            <div class="stat-label">{ "Needs Review" }</div>
                            <Link<Route> to={Route::OrganizerTalks} classes="stat-link">
                                { "Review →" }
                            </Link<Route>>
                        </div>

                        <div class="stat-card warning">
                            <div class="stat-value">{ data.unrated_talks }</div>
                            <div class="stat-label">{ "Unrated Talks" }</div>
                        </div>

                        <div class="stat-card success">
                            <div class="stat-value">
                                {
                                    if let Some(avg) = data.rating_stats.average_rating {
                                        format!("{:.1}", avg)
                                    } else {
                                        "N/A".to_string()
                                    }
                                }
                            </div>
                            <div class="stat-label">{ "Average Rating" }</div>
                        </div>
                    </div>
                </section>

                // Talk States Breakdown
                <section class="dashboard-section">
                    <h2>{ "Talk Status Distribution" }</h2>
                    <div class="stats-grid">
                        <div class="stat-card state-submitted">
                            <div class="stat-value">{ data.talks_by_state.submitted }</div>
                            <div class="stat-label">{ "Submitted" }</div>
                        </div>

                        <div class="stat-card state-pending">
                            <div class="stat-value">{ data.talks_by_state.pending }</div>
                            <div class="stat-label">{ "Pending" }</div>
                        </div>

                        <div class="stat-card state-accepted">
                            <div class="stat-value">{ data.talks_by_state.accepted }</div>
                            <div class="stat-label">{ "Accepted" }</div>
                        </div>

                        <div class="stat-card state-rejected">
                            <div class="stat-value">{ data.talks_by_state.rejected }</div>
                            <div class="stat-label">{ "Rejected" }</div>
                        </div>
                    </div>
                </section>

                // Rating Stats
                <section class="dashboard-section">
                    <h2>{ "Rating Statistics" }</h2>
                    <div class="stats-grid">
                        <div class="stat-card">
                            <div class="stat-value">{ data.rating_stats.total_ratings }</div>
                            <div class="stat-label">{ "Total Ratings" }</div>
                        </div>

                        <div class="stat-card">
                            <div class="stat-value">{ data.rating_stats.talks_with_ratings }</div>
                            <div class="stat-label">{ "Talks Rated" }</div>
                        </div>

                        <div class="stat-card">
                            <div class="stat-value">{ data.rating_stats.talks_without_ratings }</div>
                            <div class="stat-label">{ "Talks Unrated" }</div>
                        </div>
                    </div>

                    <Link<Route> to={Route::RatingsDashboard} classes="btn btn-primary">
                        { "View Detailed Rating Statistics →" }
                    </Link<Route>>
                </section>

                // Recent Submissions
                <section class="dashboard-section">
                    <h2>{ "Recent Submissions" }</h2>
                    <div class="recent-talks">
                        {
                            if data.recent_submissions.is_empty() {
                                html! { <p class="no-data">{ "No talks submitted yet." }</p> }
                            } else {
                                data.recent_submissions.iter().map(|talk| {
                                    let state_class = match talk.state {
                                        TalkState::Submitted => "submitted",
                                        TalkState::Pending => "pending",
                                        TalkState::Accepted => "accepted",
                                        TalkState::Rejected => "rejected",
                                    };

                                    html! {
                                        <div class="recent-talk-card" key={talk.id.clone()}>
                                            <div class="talk-header">
                                                <h3>{ &talk.title }</h3>
                                                <span class={classes!("state-badge", state_class)}>
                                                    {
                                                        match talk.state {
                                                            TalkState::Submitted => "Submitted",
                                                            TalkState::Pending => "Pending",
                                                            TalkState::Accepted => "Accepted",
                                                            TalkState::Rejected => "Rejected",
                                                        }
                                                    }
                                                </span>
                                            </div>
                                            <div class="talk-meta">
                                                <span class="speaker">{ format!("Speaker: {}", &talk.speaker_name) }</span>
                                                <span class="date">{ format!("Submitted: {}", &talk.submitted_at[..10]) }</span>
                                            </div>
                                            {
                                                if let Some(count) = talk.rating_count {
                                                    if count > 0 {
                                                        html! {
                                                            <div class="talk-rating">
                                                                <RatingStars
                                                                    rating={talk.average_rating.unwrap_or(0.0) as i32}
                                                                    interactive={false}
                                                                    size={"small".to_string()}
                                                                />
                                                                <span class="rating-count">
                                                                    { format!("{:.1} ({} ratings)", talk.average_rating.unwrap_or(0.0), count) }
                                                                </span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! { <div class="talk-rating unrated">{ "Not yet rated" }</div> }
                                                    }
                                                } else {
                                                    html! { <div class="talk-rating unrated">{ "Not yet rated" }</div> }
                                                }
                                            }
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        }
                    </div>
                </section>

                // Quick Actions
                <section class="dashboard-section">
                    <h2>{ "Quick Actions" }</h2>
                    <div class="quick-actions">
                        <Link<Route> to={Route::OrganizerTalks} classes="action-button primary">
                            <span class="action-icon">{ "📋" }</span>
                            <span class="action-text">{ "Review All Talks" }</span>
                        </Link<Route>>

                        <Link<Route> to={Route::RatingsDashboard} classes="action-button secondary">
                            <span class="action-icon">{ "⭐" }</span>
                            <span class="action-text">{ "View Ratings" }</span>
                        </Link<Route>>
                    </div>
                </section>
            </div>
        }
    } else {
        html! {
            <div class="organizer-dashboard">
                <div class="no-data">{ "No data available" }</div>
            </div>
        }
    }
}
