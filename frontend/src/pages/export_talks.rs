use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{Blob, Url};
use crate::services::auth::AuthService;

#[function_component(ExportTalks)]
pub fn export_talks() -> Html {
    let error = use_state(|| None::<String>);
    let exporting = use_state(|| false);
    let state_filter = use_state(|| String::from("all"));

    let on_export = {
        let exporting = exporting.clone();
        let error = error.clone();
        let state_filter = state_filter.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            let exporting = exporting.clone();
            let error = error.clone();
            let state_filter_val = (*state_filter).clone();

            wasm_bindgen_futures::spawn_local(async move {
                exporting.set(true);
                error.set(None);

                let token = match AuthService::get_token() {
                    Some(t) => t,
                    None => {
                        error.set(Some("Not authenticated".to_string()));
                        exporting.set(false);
                        return;
                    }
                };

                // Build URL with query parameters
                let url = if state_filter_val == "all" {
                    "/api/export/talks".to_string()
                } else {
                    format!("/api/export/talks?state={}", state_filter_val)
                };

                match Request::get(&url)
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            match response.text().await {
                                Ok(json_text) => {
                                    // Create blob and trigger download
                                    let blob_parts = js_sys::Array::new();
                                    blob_parts.push(&wasm_bindgen::JsValue::from_str(&json_text));

                                    let mut blob_options = web_sys::BlobPropertyBag::new();
                                    blob_options.set_type("application/json");

                                    match Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options) {
                                        Ok(blob) => {
                                            match Url::create_object_url_with_blob(&blob) {
                                                Ok(url) => {
                                                    // Create download link
                                                    let window = web_sys::window().unwrap();
                                                    let document = window.document().unwrap();
                                                    let a = document
                                                        .create_element("a")
                                                        .unwrap()
                                                        .dyn_into::<web_sys::HtmlAnchorElement>()
                                                        .unwrap();

                                                    let filename = if state_filter_val == "all" {
                                                        "talks_export.json"
                                                    } else {
                                                        &format!("talks_export_{}.json", state_filter_val)
                                                    };

                                                    a.set_href(&url);
                                                    a.set_download(filename);
                                                    a.click();

                                                    // Clean up
                                                    let _ = Url::revoke_object_url(&url);
                                                }
                                                Err(_) => {
                                                    error.set(Some("Failed to create download URL".to_string()));
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            error.set(Some("Failed to create blob".to_string()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    error.set(Some(format!("Failed to read response: {}", e)));
                                }
                            }
                        } else {
                            error.set(Some(format!("Export failed: {}", response.status())));
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Request failed: {}", e)));
                    }
                }

                exporting.set(false);
            });
        })
    };

    html! {
        <div class="export-talks-container">
            <div class="page-header">
                <h1>{ "Export Talks for AI Analysis" }</h1>
            </div>

            <div class="export-info">
                <p>
                    { "Export talk submissions in JSON format for analysis with AI tools like Claude or ChatGPT. " }
                    { "The export includes talk details, speaker information, labels, and ratings." }
                </p>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            <div class="export-form">
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
                        disabled={*exporting}
                    >
                        <option value="all">{ "All Talks" }</option>
                        <option value="submitted">{ "Submitted Only" }</option>
                        <option value="pending">{ "Pending Only" }</option>
                        <option value="accepted">{ "Accepted Only" }</option>
                        <option value="rejected">{ "Rejected Only" }</option>
                    </select>
                </div>

                <div class="form-group">
                    <button
                        onclick={on_export}
                        class="btn-primary"
                        disabled={*exporting}
                    >
                        { if *exporting { "Exporting..." } else { "Export as JSON" } }
                    </button>
                </div>
            </div>

            <div class="export-tips">
                <h3>{ "Tips for AI Analysis" }</h3>
                <ul>
                    <li>{ "Upload the exported JSON file to Claude or ChatGPT" }</li>
                    <li>{ "Ask the AI to analyze themes, categorize talks, or suggest labels" }</li>
                    <li>{ "Request recommendations for talk selection based on diversity and topics" }</li>
                    <li>{ "Generate summary statistics or identify trends in submissions" }</li>
                </ul>
            </div>
        </div>
    }
}
