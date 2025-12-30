use yew::prelude::*;
use crate::{
    services::{
        bulk_email::BulkEmailService,
        email_templates::EmailTemplateService,
        talks::TalkService,
    },
    types::{BulkEmailRequest, EmailTemplate, Talk, TalkState, BulkEmailResponse},
};

#[function_component(BulkEmail)]
pub fn bulk_email() -> Html {
    let templates = use_state(|| Vec::<EmailTemplate>::new());
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<BulkEmailResponse>);
    let sending = use_state(|| false);

    // Recipient selection
    let filter_mode = use_state(|| String::from("all"));
    let selected_states = use_state(|| Vec::<TalkState>::new());
    let selected_talk_ids = use_state(|| Vec::<String>::new());

    // Email composition
    let use_template = use_state(|| false);
    let selected_template_id = use_state(|| None::<String>);
    let custom_subject = use_state(|| String::new());
    let custom_body = use_state(|| String::new());
    let additional_message = use_state(|| String::new());

    // Fetch templates and talks on mount
    {
        let templates = templates.clone();
        let talks = talks.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);

                // Fetch email templates
                match EmailTemplateService::list_email_templates().await {
                    Ok(data) => templates.set(data),
                    Err(e) => {
                        error.set(Some(format!("Failed to load templates: {}", e)));
                    }
                }

                // Fetch all talks (for organizer)
                match TalkService::list_all_talks(None).await {
                    Ok(data) => talks.set(data),
                    Err(e) => {
                        error.set(Some(format!("Failed to load talks: {}", e)));
                    }
                }

                loading.set(false);
            });
            || ()
        });
    }

    // Submit handler
    let on_submit = {
        let sending = sending.clone();
        let error = error.clone();
        let success = success.clone();
        let filter_mode = filter_mode.clone();
        let selected_states = selected_states.clone();
        let selected_talk_ids = selected_talk_ids.clone();
        let use_template = use_template.clone();
        let selected_template_id = selected_template_id.clone();
        let custom_subject = custom_subject.clone();
        let custom_body = custom_body.clone();
        let additional_message = additional_message.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let filter_mode_val = (*filter_mode).clone();
            let selected_states_val = (*selected_states).clone();
            let selected_talk_ids_val = (*selected_talk_ids).clone();
            let use_template_val = *use_template;
            let template_id_val = (*selected_template_id).clone();
            let subject_val = (*custom_subject).clone();
            let body_val = (*custom_body).clone();
            let additional_msg_val = (*additional_message).clone();
            let sending = sending.clone();
            let error = error.clone();
            let success = success.clone();

            wasm_bindgen_futures::spawn_local(async move {
                sending.set(true);
                error.set(None);
                success.set(None);

                let request = BulkEmailRequest {
                    filter_by_state: if filter_mode_val == "state" && !selected_states_val.is_empty() {
                        Some(selected_states_val)
                    } else {
                        None
                    },
                    talk_ids: if filter_mode_val == "specific" && !selected_talk_ids_val.is_empty() {
                        Some(selected_talk_ids_val)
                    } else {
                        None
                    },
                    template_id: if use_template_val { template_id_val } else { None },
                    custom_subject: if !use_template_val && !subject_val.is_empty() {
                        Some(subject_val)
                    } else {
                        None
                    },
                    custom_body: if !use_template_val && !body_val.is_empty() {
                        Some(body_val)
                    } else {
                        None
                    },
                    additional_message: if !additional_msg_val.is_empty() {
                        Some(additional_msg_val)
                    } else {
                        None
                    },
                };

                match BulkEmailService::send_bulk_email(request).await {
                    Ok(response) => {
                        success.set(Some(response));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to send emails: {}", e)));
                    }
                }

                sending.set(false);
            });
        })
    };

    // State toggle handlers
    let create_state_toggle = |state: TalkState| {
        let selected_states = selected_states.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut states = (*selected_states).clone();
            if input.checked() {
                if !states.contains(&state) {
                    states.push(state.clone());
                }
            } else {
                states.retain(|s| s != &state);
            }
            selected_states.set(states);
        })
    };

    html! {
        <div class="bulk-email-container">
            <div class="page-header">
                <h1>{ "Send Bulk Email" }</h1>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if let Some(result) = (*success).as_ref() {
                <div class="success-message">
                    <h3>{ "Email Sent Successfully!" }</h3>
                    <p>{ format!("Emails sent: {}", result.emails_sent) }</p>
                    if result.failed_emails > 0 {
                        <p class="warning">{ format!("Failed emails: {}", result.failed_emails) }</p>
                        if !result.errors.is_empty() {
                            <details>
                                <summary>{ "Show errors" }</summary>
                                <ul>
                                    { for result.errors.iter().map(|error| html! {
                                        <li>{ error }</li>
                                    })}
                                </ul>
                            </details>
                        }
                    }
                </div>
            }

            if *loading {
                <div class="loading">{ "Loading..." }</div>
            } else {
                <form onsubmit={on_submit}>
                    <div class="form-section">
                        <h2>{ "1. Select Recipients" }</h2>

                        <div class="form-group">
                            <label>{ "Filter Mode" }</label>
                            <select
                                value={(*filter_mode).clone()}
                                onchange={Callback::from({
                                    let filter_mode = filter_mode.clone();
                                    move |e: Event| {
                                        let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                        filter_mode.set(input.value());
                                    }
                                })}
                            >
                                <option value="all">{ "All speakers" }</option>
                                <option value="state">{ "Filter by talk state" }</option>
                                <option value="specific">{ "Specific talks" }</option>
                            </select>
                        </div>

                        if *filter_mode == "state" {
                            <div class="form-group">
                                <label>{ "Select Talk States" }</label>
                                <div class="checkbox-group">
                                    <label>
                                        <input
                                            type="checkbox"
                                            onchange={create_state_toggle(TalkState::Submitted)}
                                        />
                                        { " Submitted" }
                                    </label>
                                    <label>
                                        <input
                                            type="checkbox"
                                            onchange={create_state_toggle(TalkState::Pending)}
                                        />
                                        { " Pending" }
                                    </label>
                                    <label>
                                        <input
                                            type="checkbox"
                                            onchange={create_state_toggle(TalkState::Accepted)}
                                        />
                                        { " Accepted" }
                                    </label>
                                    <label>
                                        <input
                                            type="checkbox"
                                            onchange={create_state_toggle(TalkState::Rejected)}
                                        />
                                        { " Rejected" }
                                    </label>
                                </div>
                            </div>
                        }

                        if *filter_mode == "specific" {
                            <div class="form-group">
                                <label>{ "Select Talks" }</label>
                                <div class="talk-selection-list">
                                    { for talks.iter().map(|talk| {
                                        let talk_id = talk.id.clone();
                                        let selected_talk_ids = selected_talk_ids.clone();
                                        let is_selected = selected_talk_ids.contains(&talk.id);

                                        html! {
                                            <label key={talk.id.clone()}>
                                                <input
                                                    type="checkbox"
                                                    checked={is_selected}
                                                    onchange={Callback::from(move |e: Event| {
                                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                        let mut ids = (*selected_talk_ids).clone();
                                                        if input.checked() {
                                                            if !ids.contains(&talk_id) {
                                                                ids.push(talk_id.clone());
                                                            }
                                                        } else {
                                                            ids.retain(|id| id != &talk_id);
                                                        }
                                                        selected_talk_ids.set(ids);
                                                    })}
                                                />
                                                { format!(" {} - {}", talk.title, talk.speaker_name) }
                                            </label>
                                        }
                                    })}
                                </div>
                            </div>
                        }
                    </div>

                    <div class="form-section">
                        <h2>{ "2. Compose Email" }</h2>

                        <div class="form-group">
                            <label>
                                <input
                                    type="checkbox"
                                    checked={*use_template}
                                    onchange={Callback::from({
                                        let use_template = use_template.clone();
                                        move |e: Event| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            use_template.set(input.checked());
                                        }
                                    })}
                                />
                                { " Use email template" }
                            </label>
                        </div>

                        if *use_template {
                            <div class="form-group">
                                <label>{ "Select Template" }</label>
                                <select
                                    value={(*selected_template_id).clone().unwrap_or_default()}
                                    onchange={Callback::from({
                                        let selected_template_id = selected_template_id.clone();
                                        move |e: Event| {
                                            let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                            let value = input.value();
                                            selected_template_id.set(if value.is_empty() {
                                                None
                                            } else {
                                                Some(value)
                                            });
                                        }
                                    })}
                                    required=true
                                >
                                    <option value="">{ "Select a template" }</option>
                                    { for templates.iter().map(|template| html! {
                                        <option value={template.id.clone()}>
                                            { format!("{} - {}", template.name, template.template_type) }
                                        </option>
                                    })}
                                </select>
                            </div>
                        } else {
                            <div class="form-group">
                                <label>{ "Email Subject *" }</label>
                                <input
                                    type="text"
                                    value={(*custom_subject).clone()}
                                    oninput={Callback::from({
                                        let custom_subject = custom_subject.clone();
                                        move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            custom_subject.set(input.value());
                                        }
                                    })}
                                    required=true
                                    placeholder="e.g. Important update about your talk"
                                />
                            </div>

                            <div class="form-group">
                                <label>{ "Email Body *" }</label>
                                <textarea
                                    value={(*custom_body).clone()}
                                    oninput={Callback::from({
                                        let custom_body = custom_body.clone();
                                        move |e: InputEvent| {
                                            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                            custom_body.set(input.value());
                                        }
                                    })}
                                    required=true
                                    rows="10"
                                    placeholder="Your email message..."
                                />
                                <small>{ "Tip: You can use variables like {{speaker_name}}, {{talk_title}}, etc." }</small>
                            </div>
                        }

                        <div class="form-group">
                            <label>{ "Additional Message (optional)" }</label>
                            <textarea
                                value={(*additional_message).clone()}
                                oninput={Callback::from({
                                    let additional_message = additional_message.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                        additional_message.set(input.value());
                                    }
                                })}
                                rows="3"
                                placeholder="Optional additional context or notes..."
                            />
                        </div>
                    </div>

                    <div class="form-actions">
                        <button
                            type="submit"
                            class="btn-primary"
                            disabled={*sending}
                        >
                            { if *sending { "Sending..." } else { "Send Bulk Email" } }
                        </button>
                    </div>
                </form>
            }
        </div>
    }
}
