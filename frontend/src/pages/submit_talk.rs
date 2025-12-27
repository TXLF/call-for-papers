use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::{File, HtmlInputElement, HtmlTextAreaElement};

use crate::{
    app::Route,
    services::talks::TalkService,
    types::{CreateTalkRequest, Talk},
};

#[function_component(SubmitTalk)]
pub fn submit_talk() -> Html {
    let navigator = use_navigator().unwrap();
    let title = use_state(|| String::new());
    let short_summary = use_state(|| String::new());
    let long_description = use_state(|| String::new());
    let slides_file = use_state(|| None::<File>);
    let created_talk = use_state(|| None::<Talk>);
    let error = use_state(|| None::<String>);
    let success = use_state(|| false);
    let loading = use_state(|| false);
    let uploading_slides = use_state(|| false);

    let title_clone = title.clone();
    let on_title_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        title_clone.set(input.value());
    });

    let short_summary_clone = short_summary.clone();
    let on_short_summary_change = Callback::from(move |e: Event| {
        let textarea: HtmlTextAreaElement = e.target_unchecked_into();
        short_summary_clone.set(textarea.value());
    });

    let long_description_clone = long_description.clone();
    let on_long_description_change = Callback::from(move |e: Event| {
        let textarea: HtmlTextAreaElement = e.target_unchecked_into();
        long_description_clone.set(textarea.value());
    });

    let slides_file_clone = slides_file.clone();
    let on_file_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                slides_file_clone.set(Some(file));
            }
        }
    });

    let on_submit = {
        let title = title.clone();
        let short_summary = short_summary.clone();
        let long_description = long_description.clone();
        let slides_file = slides_file.clone();
        let created_talk = created_talk.clone();
        let error = error.clone();
        let success = success.clone();
        let loading = loading.clone();
        let uploading_slides = uploading_slides.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let title_val = (*title).clone();
            let short_summary_val = (*short_summary).clone();
            let long_description_val = (*long_description).clone();
            let slides_file_opt = (*slides_file).clone();
            let created_talk = created_talk.clone();
            let error = error.clone();
            let success = success.clone();
            let loading = loading.clone();
            let uploading_slides = uploading_slides.clone();

            if title_val.trim().is_empty() || short_summary_val.trim().is_empty() {
                error.set(Some("Title and short summary are required".to_string()));
                return;
            }

            if title_val.len() > 500 {
                error.set(Some("Title must be 500 characters or less".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);
            success.set(false);

            wasm_bindgen_futures::spawn_local(async move {
                let long_desc = if long_description_val.trim().is_empty() {
                    None
                } else {
                    Some(long_description_val.trim().to_string())
                };

                let request = CreateTalkRequest {
                    title: title_val.trim().to_string(),
                    short_summary: short_summary_val.trim().to_string(),
                    long_description: long_desc,
                };

                match TalkService::create_talk(request).await {
                    Ok(talk) => {
                        loading.set(false);

                        // Upload slides if file was selected
                        if let Some(file) = slides_file_opt {
                            uploading_slides.set(true);
                            match TalkService::upload_slides(&talk.id, file).await {
                                Ok(updated_talk) => {
                                    created_talk.set(Some(updated_talk));
                                    success.set(true);
                                    uploading_slides.set(false);
                                }
                                Err(e) => {
                                    // Talk was created but slide upload failed
                                    created_talk.set(Some(talk));
                                    error.set(Some(format!("Talk created, but slide upload failed: {}", e)));
                                    success.set(true);
                                    uploading_slides.set(false);
                                }
                            }
                        } else {
                            created_talk.set(Some(talk));
                            success.set(true);
                        }
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let on_view_talks = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::MyTalks);
        })
    };

    html! {
        <div class="form-container">
            <div class="form-card">
                <h2>{ "Submit a Talk" }</h2>

                {
                    if *success {
                        html! {
                            <div class="success-message">
                                <p>{ "Your talk has been submitted successfully!" }</p>
                                {
                                    if *uploading_slides {
                                        html! { <p>{ "Uploading slides..." }</p> }
                                    } else if let Some(talk) = (*created_talk).as_ref() {
                                        if talk.slides_url.is_some() {
                                            html! { <p>{ "Slides uploaded successfully!" }</p> }
                                        } else {
                                            html! {}
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                                <button onclick={on_view_talks}>{ "View My Talks" }</button>
                            </div>
                        }
                    } else {
                        html! {
                            <form onsubmit={on_submit}>
                                <div class="form-group">
                                    <label for="title">
                                        { "Title " }
                                        <span class="required">{ "*" }</span>
                                    </label>
                                    <input
                                        type="text"
                                        id="title"
                                        value={(*title).clone()}
                                        onchange={on_title_change}
                                        disabled={*loading}
                                        placeholder="Enter your talk title (max 500 characters)"
                                        maxlength="500"
                                        required=true
                                    />
                                    <small class="char-count">
                                        { format!("{}/500 characters", title.len()) }
                                    </small>
                                </div>

                                <div class="form-group">
                                    <label for="short_summary">
                                        { "Short Summary " }
                                        <span class="required">{ "*" }</span>
                                    </label>
                                    <textarea
                                        id="short_summary"
                                        value={(*short_summary).clone()}
                                        onchange={on_short_summary_change}
                                        disabled={*loading}
                                        placeholder="Brief description of your talk (2-3 sentences)"
                                        rows="4"
                                        required=true
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="long_description">
                                        { "Long Description " }
                                        <span class="optional">{ "(Optional)" }</span>
                                    </label>
                                    <textarea
                                        id="long_description"
                                        value={(*long_description).clone()}
                                        onchange={on_long_description_change}
                                        disabled={*loading}
                                        placeholder="Detailed description, outline, or additional information"
                                        rows="8"
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="slides">
                                        { "Slides " }
                                        <span class="optional">{ "(Optional)" }</span>
                                    </label>
                                    <input
                                        type="file"
                                        id="slides"
                                        onchange={on_file_change}
                                        disabled={*loading}
                                        accept=".pdf,.ppt,.pptx,.key,.odp"
                                    />
                                    <small class="file-note">
                                        { "Accepted formats: PDF, PPT, PPTX, KEY, ODP (max 50MB)" }
                                    </small>
                                    {
                                        if let Some(file) = (*slides_file).as_ref() {
                                            html! {
                                                <small class="file-selected">
                                                    { format!("Selected: {}", file.name()) }
                                                </small>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>

                                {
                                    if let Some(err) = (*error).as_ref() {
                                        html! {
                                            <div class="error-message">
                                                { err }
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }

                                <button type="submit" disabled={*loading || *uploading_slides}>
                                    {
                                        if *loading {
                                            "Submitting..."
                                        } else if *uploading_slides {
                                            "Uploading slides..."
                                        } else {
                                            "Submit Talk"
                                        }
                                    }
                                </button>
                            </form>
                        }
                    }
                }
            </div>
        </div>
    }
}
