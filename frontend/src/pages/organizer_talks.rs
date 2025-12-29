use yew::prelude::*;
use web_sys::HtmlSelectElement;

use crate::{
    components::LabelBadge,
    services::talks::TalkService,
    types::Talk,
};

#[function_component(OrganizerTalks)]
pub fn organizer_talks() -> Html {
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let selected_filter = use_state(|| String::from(""));

    // Fetch talks whenever filter changes
    {
        let talks = talks.clone();
        let loading = loading.clone();
        let error = error.clone();
        let filter = (*selected_filter).clone();

        use_effect_with(selected_filter.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);

                let state_filter = if filter.is_empty() {
                    None
                } else {
                    Some(filter)
                };

                match TalkService::list_all_talks(state_filter).await {
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
