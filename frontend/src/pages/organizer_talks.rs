use yew::prelude::*;
use web_sys::HtmlSelectElement;
use wasm_bindgen_futures::spawn_local;
use std::collections::HashMap;

use crate::{
    components::{LabelBadge, RatingForm, RatingStars},
    services::{talks::TalkService, ratings::RatingService},
    types::{Talk, Rating},
};

#[function_component(OrganizerTalks)]
pub fn organizer_talks() -> Html {
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let selected_filter = use_state(|| String::from(""));
    let my_ratings = use_state(|| HashMap::<String, Rating>::new());
    let all_ratings = use_state(|| HashMap::<String, Vec<Rating>>::new());
    let expanded_talk_id = use_state(|| None::<String>);

    // Fetch talks whenever filter changes
    {
        let talks = talks.clone();
        let loading = loading.clone();
        let error = error.clone();
        let my_ratings = my_ratings.clone();
        let filter = (*selected_filter).clone();

        use_effect_with(selected_filter.clone(), move |_| {
            spawn_local(async move {
                loading.set(true);
                error.set(None);

                let state_filter = if filter.is_empty() {
                    None
                } else {
                    Some(filter)
                };

                match TalkService::list_all_talks(state_filter).await {
                    Ok(fetched_talks) => {
                        talks.set(fetched_talks.clone());

                        // Fetch my ratings for all talks
                        let mut ratings_map = HashMap::new();
                        for talk in fetched_talks.iter() {
                            if let Ok(Some(rating)) = RatingService::get_my_rating(&talk.id).await {
                                ratings_map.insert(talk.id.clone(), rating);
                            }
                        }
                        my_ratings.set(ratings_map);

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

    let on_filter_change = {
        let selected_filter = selected_filter.clone();
        Callback::from(move |e: Event| {
            let select: HtmlSelectElement = e.target_unchecked_into();
            selected_filter.set(select.value());
        })
    };

    html! {
        <div class="talks-container">
            <div class="talks-header">
                <h2>{ "All Talk Submissions" }</h2>
                <div class="filter-group">
                    <label for="state-filter">{ "Filter by state: " }</label>
                    <select
                        id="state-filter"
                        class="filter-dropdown"
                        onchange={on_filter_change}
                        value={(*selected_filter).clone()}
                    >
                        <option value="">{ "All" }</option>
                        <option value="submitted">{ "Submitted" }</option>
                        <option value="pending">{ "Pending" }</option>
                        <option value="accepted">{ "Accepted" }</option>
                        <option value="rejected">{ "Rejected" }</option>
                    </select>
                </div>
            </div>

            {
                if *loading {
                    html! {
                        <div class="loading">{ "Loading talks..." }</div>
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
                            <p>{ "No talks found" }</p>
                            {
                                if !(*selected_filter).is_empty() {
                                    html! {
                                        <p>{ "Try changing the filter or removing it to see all talks." }</p>
                                    }
                                } else {
                                    html! {
                                        <p>{ "No talks have been submitted yet." }</p>
                                    }
                                }
                            }
                        </div>
                    }
                } else {
                    html! {
                        <div class="talks-list">
                            {
                                for talks.iter().map(|talk| {
                                    let state_class = format!("state-{:?}", talk.state).to_lowercase();
                                    let talk_id = talk.id.clone();
                                    let existing_rating = my_ratings.get(&talk_id).cloned();
                                    let is_expanded = expanded_talk_id.as_ref() == Some(&talk_id);

                                    let my_ratings_clone = my_ratings.clone();
                                    let all_ratings_clone = all_ratings.clone();
                                    let talk_id_for_success = talk_id.clone();
                                    let on_rating_success = Callback::from(move |new_rating: Rating| {
                                        let mut ratings_map = (*my_ratings_clone).clone();
                                        ratings_map.insert(talk_id_for_success.clone(), new_rating);
                                        my_ratings_clone.set(ratings_map);
                                    });

                                    let my_ratings_clone = my_ratings.clone();
                                    let talk_id_for_delete = talk_id.clone();
                                    let on_rating_delete = Callback::from(move |_| {
                                        let mut ratings_map = (*my_ratings_clone).clone();
                                        ratings_map.remove(&talk_id_for_delete);
                                        my_ratings_clone.set(ratings_map);
                                    });

                                    let expanded_talk_id_clone = expanded_talk_id.clone();
                                    let all_ratings_clone = all_ratings.clone();
                                    let talk_id_for_toggle = talk_id.clone();
                                    let on_toggle_ratings = Callback::from(move |_| {
                                        let is_currently_expanded = expanded_talk_id_clone.as_ref() == Some(&talk_id_for_toggle);
                                        if is_currently_expanded {
                                            expanded_talk_id_clone.set(None);
                                        } else {
                                            expanded_talk_id_clone.set(Some(talk_id_for_toggle.clone()));
                                            // Fetch all ratings for this talk
                                            let talk_id_for_fetch = talk_id_for_toggle.clone();
                                            let all_ratings_clone_inner = all_ratings_clone.clone();
                                            spawn_local(async move {
                                                if let Ok(ratings) = RatingService::get_talk_ratings(&talk_id_for_fetch).await {
                                                    let mut all_ratings_map = (*all_ratings_clone_inner).clone();
                                                    all_ratings_map.insert(talk_id_for_fetch, ratings);
                                                    all_ratings_clone_inner.set(all_ratings_map);
                                                }
                                            });
                                        }
                                    });

                                    let talk_ratings = all_ratings.get(&talk_id);
                                    let average_rating = talk_ratings.as_ref().and_then(|ratings| {
                                        if ratings.is_empty() {
                                            None
                                        } else {
                                            let sum: i32 = ratings.iter().map(|r| r.rating).sum();
                                            Some(sum as f64 / ratings.len() as f64)
                                        }
                                    });

                                    html! {
                                        <div class="talk-card" key={talk.id.clone()}>
                                            <div class="talk-header">
                                                <h3>{ &talk.title }</h3>
                                                <span class={classes!("talk-status", state_class)}>
                                                    { format!("{:?}", talk.state) }
                                                </span>
                                            </div>

                                            <div class="speaker-info">
                                                <strong>{ "Speaker: " }</strong>
                                                { &talk.speaker_name }
                                                <span class="speaker-email">
                                                    { " <" }{ &talk.speaker_email }{ ">" }
                                                </span>
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

                                            <div class="talk-meta">
                                                <small>{ format!("Submitted: {}", talk.submitted_at) }</small>
                                            </div>

                                            <div class="rating-section">
                                                <h3>{ "Rate This Talk" }</h3>
                                                <RatingForm
                                                    talk_id={talk_id.clone()}
                                                    existing_rating={existing_rating}
                                                    on_success={on_rating_success}
                                                    on_delete={on_rating_delete}
                                                />

                                                <div style="margin-top: 1rem;">
                                                    <button
                                                        class="btn-primary"
                                                        onclick={on_toggle_ratings}
                                                        style="font-size: 0.9rem; padding: 0.5rem 1rem;"
                                                    >
                                                        {
                                                            if is_expanded {
                                                                "Hide All Ratings"
                                                            } else {
                                                                "View All Ratings"
                                                            }
                                                        }
                                                    </button>
                                                </div>

                                                {
                                                    if is_expanded {
                                                        html! {
                                                            <>
                                                                {
                                                                    if let Some(ratings) = talk_ratings {
                                                                        if ratings.is_empty() {
                                                                            html! {
                                                                                <p style="margin-top: 1rem; color: #666;">
                                                                                    { "No ratings yet for this talk." }
                                                                                </p>
                                                                            }
                                                                        } else {
                                                                            html! {
                                                                                <>
                                                                                    {
                                                                                        if let Some(avg) = average_rating {
                                                                                            html! {
                                                                                                <div class="rating-stats">
                                                                                                    <div class="average-rating">
                                                                                                        <span class="average-rating-number">
                                                                                                            { format!("{:.1}", avg) }
                                                                                                        </span>
                                                                                                        <RatingStars
                                                                                                            rating={avg.round() as i32}
                                                                                                            interactive={false}
                                                                                                            size="small"
                                                                                                        />
                                                                                                    </div>
                                                                                                    <span class="rating-count">
                                                                                                        { format!("({} rating{})", ratings.len(), if ratings.len() == 1 { "" } else { "s" }) }
                                                                                                    </span>
                                                                                                </div>
                                                                                            }
                                                                                        } else {
                                                                                            html! {}
                                                                                        }
                                                                                    }
                                                                                    <div class="ratings-list">
                                                                                        {
                                                                                            for ratings.iter().map(|rating| {
                                                                                                html! {
                                                                                                    <div class="rating-item" key={rating.id.clone()}>
                                                                                                        <div class="rating-header">
                                                                                                            <div>
                                                                                                                <div class="rating-organizer">
                                                                                                                    { &rating.organizer_name }
                                                                                                                </div>
                                                                                                                <RatingStars
                                                                                                                    rating={rating.rating}
                                                                                                                    interactive={false}
                                                                                                                    size="small"
                                                                                                                />
                                                                                                            </div>
                                                                                                            <span class="rating-date">
                                                                                                                { &rating.created_at }
                                                                                                            </span>
                                                                                                        </div>
                                                                                                        {
                                                                                                            if let Some(notes) = &rating.notes {
                                                                                                                html! {
                                                                                                                    <div class="rating-notes">
                                                                                                                        { notes }
                                                                                                                    </div>
                                                                                                                }
                                                                                                            } else {
                                                                                                                html! {}
                                                                                                            }
                                                                                                        }
                                                                                                    </div>
                                                                                                }
                                                                                            })
                                                                                        }
                                                                                    </div>
                                                                                </>
                                                                            }
                                                                        }
                                                                    } else {
                                                                        html! {
                                                                            <p style="margin-top: 1rem; color: #666;">
                                                                                { "Loading ratings..." }
                                                                            </p>
                                                                        }
                                                                    }
                                                                }
                                                            </>
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
