use yew::prelude::*;
use yew_router::prelude::*;

use crate::{app::Route, services::talks::TalkService, types::Talk};

#[function_component(MyTalks)]
pub fn my_talks() -> Html {
    let navigator = use_navigator().unwrap();
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

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

    html! {
        <div class="talks-container">
            <div class="talks-header">
                <h2>{ "My Talk Submissions" }</h2>
                <button onclick={on_submit_new.clone()} class="btn-primary">
                    { "Submit New Talk" }
                </button>
            </div>

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
                                    html! {
                                        <div class="talk-card" key={talk.id.clone()}>
                                            <div class="talk-header">
                                                <h3>{ &talk.title }</h3>
                                                <span class={classes!("talk-status", state_class)}>
                                                    { format!("{:?}", talk.state) }
                                                </span>
                                            </div>
                                            <p class="talk-summary">{ &talk.short_summary }</p>
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
