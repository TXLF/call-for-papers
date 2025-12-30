use yew::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::{services::auth::AuthService, types::Label};

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct AutoTagResponse {
    suggested_labels: Vec<String>,
    existing_labels: Vec<String>,
    new_labels: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CreateLabelsRequest {
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CreateLabelsResponse {
    created: Vec<Label>,
    skipped: Vec<String>,
}

#[function_component(AIAutoTag)]
pub fn ai_auto_tag() -> Html {
    let analyzing = use_state(|| false);
    let creating = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);
    let suggestions = use_state(|| None::<AutoTagResponse>);
    let state_filter = use_state(|| String::from("all"));
    let ai_provider = use_state(|| String::from("claude"));
    let selected_labels = use_state(|| Vec::<String>::new());

    let on_analyze = {
        let analyzing = analyzing.clone();
        let error = error.clone();
        let success = success.clone();
        let suggestions = suggestions.clone();
        let state_filter = state_filter.clone();
        let ai_provider = ai_provider.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            let analyzing = analyzing.clone();
            let error = error.clone();
            let success = success.clone();
            let suggestions = suggestions.clone();
            let state_filter_val = (*state_filter).clone();
            let provider_val = (*ai_provider).clone();

            wasm_bindgen_futures::spawn_local(async move {
                analyzing.set(true);
                error.set(None);
                success.set(None);

                let token = match AuthService::get_token() {
                    Some(t) => t,
                    None => {
                        error.set(Some("Not authenticated".to_string()));
                        analyzing.set(false);
                        return;
                    }
                };

                let url = if state_filter_val == "all" {
                    format!("/api/ai/auto-tag?provider={}", provider_val)
                } else {
                    format!("/api/ai/auto-tag?state={}&provider={}", state_filter_val, provider_val)
                };

                match Request::get(&url)
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<AutoTagResponse>().await {
                                Ok(result) => {
                                    suggestions.set(Some(result));
                                    let provider_name = if provider_val == "claude" { "Claude" } else { "ChatGPT" };
                                    success.set(Some(format!("Analysis complete using {}! Review the suggested labels below.", provider_name)));
                                }
                                Err(e) => {
                                    error.set(Some(format!("Failed to parse response: {}", e)));
                                }
                            }
                        } else {
                            match response.text().await {
                                Ok(text) => error.set(Some(format!("Analysis failed: {}", text))),
                                Err(e) => error.set(Some(format!("Analysis failed: {}", e))),
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Request failed: {}", e)));
                    }
                }

                analyzing.set(false);
            });
        })
    };

    let on_create_labels = {
        let creating = creating.clone();
        let error = error.clone();
        let success = success.clone();
        let selected_labels = selected_labels.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            let creating = creating.clone();
            let error = error.clone();
            let success = success.clone();
            let labels_to_create = (*selected_labels).clone();

            if labels_to_create.is_empty() {
                error.set(Some("Please select at least one label to create".to_string()));
                return;
            }

            wasm_bindgen_futures::spawn_local(async move {
                creating.set(true);
                error.set(None);

                let token = match AuthService::get_token() {
                    Some(t) => t,
                    None => {
                        error.set(Some("Not authenticated".to_string()));
                        creating.set(false);
                        return;
                    }
                };

                let request = CreateLabelsRequest {
                    labels: labels_to_create.clone(),
                };

                match Request::post("/api/ai/create-labels")
                    .header("Authorization", &format!("Bearer {}", token))
                    .json(&request)
                {
                    Ok(req) => {
                        match req.send().await {
                            Ok(response) => {
                                if response.ok() {
                                    match response.json::<CreateLabelsResponse>().await {
                                        Ok(result) => {
                                            success.set(Some(format!(
                                                "Successfully created {} labels. {} labels were skipped (already exist).",
                                                result.created.len(),
                                                result.skipped.len()
                                            )));
                                        }
                                        Err(e) => {
                                            error.set(Some(format!("Failed to parse response: {}", e)));
                                        }
                                    }
                                } else {
                                    match response.text().await {
                                        Ok(text) => error.set(Some(format!("Failed to create labels: {}", text))),
                                        Err(e) => error.set(Some(format!("Failed to create labels: {}", e))),
                                    }
                                }
                            }
                            Err(e) => {
                                error.set(Some(format!("Request failed: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to serialize request: {}", e)));
                    }
                }

                creating.set(false);
            });
        })
    };

    let create_label_toggle = |label: String| {
        let selected_labels = selected_labels.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut labels = (*selected_labels).clone();
            if input.checked() {
                if !labels.contains(&label) {
                    labels.push(label.clone());
                }
            } else {
                labels.retain(|l| l != &label);
            }
            selected_labels.set(labels);
        })
    };

    html! {
        <div class="ai-auto-tag-container">
            <div class="page-header">
                <h1>{ "AI Auto-Tagging" }</h1>
            </div>

            <div class="ai-info">
                <p>
                    { "Use AI to analyze your talk submissions and suggest relevant labels for categorization. " }
                    { "The AI will analyze talk titles, summaries, and descriptions to recommend appropriate tags." }
                </p>
                <p><strong>{ "Note:" }</strong>{ " This feature requires either a Claude API key (CLAUDE_API_KEY) or OpenAI API key (OPENAI_API_KEY) to be configured." }</p>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if let Some(msg) = (*success).as_ref() {
                <div class="success-message">{ msg }</div>
            }

            <div class="analyze-section">
                <h2>{ "Step 1: Analyze Talks" }</h2>

                <div class="form-group">
                    <label>{ "AI Provider" }</label>
                    <select
                        value={(*ai_provider).clone()}
                        onchange={Callback::from({
                            let ai_provider = ai_provider.clone();
                            move |e: Event| {
                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                ai_provider.set(input.value());
                            }
                        })}
                        disabled={*analyzing}
                    >
                        <option value="claude">{ "Claude (Anthropic)" }</option>
                        <option value="openai">{ "ChatGPT (OpenAI)" }</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>{ "Filter by State" }</label>
                    <select
                        value={(*state_filter).clone()}
                        onchange={Callback::from({
                            let state_filter = state_filter.clone();
                            move |e: Event| {
                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                state_filter.set(input.value());
                            }
                        })}
                        disabled={*analyzing}
                    >
                        <option value="all">{ "All Talks" }</option>
                        <option value="submitted">{ "Submitted Only" }</option>
                        <option value="pending">{ "Pending Only" }</option>
                        <option value="accepted">{ "Accepted Only" }</option>
                    </select>
                </div>

                <button
                    onclick={on_analyze}
                    class="btn-primary"
                    disabled={*analyzing}
                >
                    { if *analyzing { "Analyzing with AI..." } else { "Analyze Talks" } }
                </button>
            </div>

            if let Some(result) = (*suggestions).as_ref() {
                <div class="results-section">
                    <h2>{ "Step 2: Review Suggested Labels" }</h2>

                    <div class="label-summary">
                        <p>{ format!("AI suggested {} labels:", result.suggested_labels.len()) }</p>
                        <ul>
                            <li>{ format!("{} already exist in your system", result.existing_labels.len()) }</li>
                            <li>{ format!("{} are new and can be created", result.new_labels.len()) }</li>
                        </ul>
                    </div>

                    if !result.existing_labels.is_empty() {
                        <div class="existing-labels">
                            <h3>{ "Existing Labels" }</h3>
                            <div class="label-chips">
                                { for result.existing_labels.iter().map(|label| html! {
                                    <span class="label-chip existing">{ label }</span>
                                })}
                            </div>
                        </div>
                    }

                    if !result.new_labels.is_empty() {
                        <div class="new-labels">
                            <h3>{ "New Labels (Select to Create)" }</h3>
                            <div class="label-checkboxes">
                                { for result.new_labels.iter().map(|label| {
                                    let label_clone = label.clone();
                                    let is_selected = (*selected_labels).contains(label);
                                    html! {
                                        <label class="label-checkbox" key={label.clone()}>
                                            <input
                                                type="checkbox"
                                                checked={is_selected}
                                                onchange={create_label_toggle(label_clone)}
                                            />
                                            { format!(" {}", label) }
                                        </label>
                                    }
                                })}
                            </div>

                            <div class="create-actions">
                                <button
                                    onclick={on_create_labels}
                                    class="btn-primary"
                                    disabled={*creating || selected_labels.is_empty()}
                                >
                                    { if *creating {
                                        "Creating Labels...".to_string()
                                    } else {
                                        format!("Create {} Selected Labels", selected_labels.len())
                                    }}
                                </button>
                            </div>
                        </div>
                    }
                </div>
            }
        </div>
    }
}
