use yew::prelude::*;
use crate::{
    services::{email_templates::EmailTemplateService, conferences::ConferenceService},
    types::{EmailTemplate, CreateEmailTemplateRequest, UpdateEmailTemplateRequest},
};

#[function_component(ManageEmailTemplates)]
pub fn manage_email_templates() -> Html {
    let templates = use_state(|| Vec::<EmailTemplate>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let editing_template = use_state(|| None::<EmailTemplate>);
    let conference_id = use_state(|| None::<String>);

    // Form state
    let name = use_state(|| String::new());
    let subject = use_state(|| String::new());
    let body = use_state(|| String::new());
    let template_type = use_state(|| String::from("custom"));
    let is_default = use_state(|| false);

    // Fetch active conference and templates on mount
    {
        let conference_id = conference_id.clone();
        let templates = templates.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);

                // Fetch active conference
                match ConferenceService::get_active_conference().await {
                    Ok(conf) => {
                        conference_id.set(Some(conf.id));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load active conference: {}", e)));
                        loading.set(false);
                        return;
                    }
                }

                // Fetch email templates
                match EmailTemplateService::list_email_templates().await {
                    Ok(data) => {
                        templates.set(data);
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

    // Submit handler (create or update)
    let on_submit = {
        let templates = templates.clone();
        let conference_id = conference_id.clone();
        let editing_template = editing_template.clone();
        let name = name.clone();
        let subject = subject.clone();
        let body = body.clone();
        let template_type = template_type.clone();
        let is_default = is_default.clone();
        let show_form = show_form.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let templates = templates.clone();
            let conf_id = (*conference_id).clone();
            let editing = (*editing_template).clone();
            let name_val = (*name).clone();
            let subject_val = (*subject).clone();
            let body_val = (*body).clone();
            let template_type_val = (*template_type).clone();
            let is_default_val = *is_default;
            let show_form = show_form.clone();
            let editing_template = editing_template.clone();
            let error = error.clone();
            let name = name.clone();
            let subject = subject.clone();
            let body = body.clone();
            let template_type = template_type.clone();
            let is_default = is_default.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Some(template) = editing {
                    // Update existing template
                    let request = UpdateEmailTemplateRequest {
                        name: Some(name_val),
                        subject: Some(subject_val),
                        body: Some(body_val),
                        is_default: Some(is_default_val),
                    };

                    match EmailTemplateService::update_email_template(&template.id, request).await {
                        Ok(updated_template) => {
                            let mut current_templates = (*templates).clone();
                            if let Some(index) = current_templates.iter().position(|t| t.id == template.id) {
                                current_templates[index] = updated_template;
                            }
                            templates.set(current_templates);
                            name.set(String::new());
                            subject.set(String::new());
                            body.set(String::new());
                            template_type.set(String::from("custom"));
                            is_default.set(false);
                            show_form.set(false);
                            editing_template.set(None);
                            error.set(None);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to update template: {}", e)));
                        }
                    }
                } else {
                    // Create new template
                    let conf_id = match conf_id {
                        Some(id) => id,
                        None => {
                            error.set(Some("No active conference found".to_string()));
                            return;
                        }
                    };

                    let request = CreateEmailTemplateRequest {
                        conference_id: conf_id,
                        template_type: template_type_val,
                        name: name_val,
                        subject: subject_val,
                        body: body_val,
                        is_default: Some(is_default_val),
                    };

                    match EmailTemplateService::create_email_template(request).await {
                        Ok(new_template) => {
                            let mut current_templates = (*templates).clone();
                            current_templates.push(new_template);
                            templates.set(current_templates);
                            name.set(String::new());
                            subject.set(String::new());
                            body.set(String::new());
                            template_type.set(String::from("custom"));
                            is_default.set(false);
                            show_form.set(false);
                            error.set(None);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to create template: {}", e)));
                        }
                    }
                }
            });
        })
    };

    // Delete template handler
    let create_delete_handler = |template_id: String| {
        let templates = templates.clone();
        let error = error.clone();

        Callback::from(move |_: MouseEvent| {
            let templates = templates.clone();
            let template_id = template_id.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match EmailTemplateService::delete_email_template(&template_id).await {
                    Ok(_) => {
                        let mut current_templates = (*templates).clone();
                        current_templates.retain(|t| t.id != template_id);
                        templates.set(current_templates);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to delete template: {}", e)));
                    }
                }
            });
        })
    };

    // Edit button handler
    let create_edit_handler = |template: EmailTemplate| {
        let show_form = show_form.clone();
        let editing_template = editing_template.clone();
        let name = name.clone();
        let subject = subject.clone();
        let body = body.clone();
        let template_type = template_type.clone();
        let is_default = is_default.clone();

        Callback::from(move |_: MouseEvent| {
            name.set(template.name.clone());
            subject.set(template.subject.clone());
            body.set(template.body.clone());
            template_type.set(template.template_type.clone());
            is_default.set(template.is_default);
            editing_template.set(Some(template.clone()));
            show_form.set(true);
        })
    };

    let show_form_display = *show_form;
    let is_editing = editing_template.is_some();

    let toggle_form = {
        let show_form = show_form.clone();
        let editing_template = editing_template.clone();
        let name = name.clone();
        let subject = subject.clone();
        let body = body.clone();
        let template_type = template_type.clone();
        let is_default = is_default.clone();

        Callback::from(move |_| {
            if *show_form {
                // Cancel - clear form
                name.set(String::new());
                subject.set(String::new());
                body.set(String::new());
                template_type.set(String::from("custom"));
                is_default.set(false);
                editing_template.set(None);
                show_form.set(false);
            } else {
                // Open create form
                name.set(String::new());
                subject.set(String::new());
                body.set(String::new());
                template_type.set(String::from("custom"));
                is_default.set(false);
                editing_template.set(None);
                show_form.set(true);
            }
        })
    };

    html! {
        <div class="manage-email-templates-container">
            <div class="page-header">
                <h1>{ "Manage Email Templates" }</h1>
                <button
                    onclick={toggle_form}
                    class="btn-primary"
                >
                    { if show_form_display { "Cancel" } else { "Add Template" } }
                </button>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if *show_form {
                <div class="create-form-card">
                    <h2>{ if is_editing { "Edit Email Template" } else { "Create New Email Template" } }</h2>

                    <div class="template-variables-help">
                        <strong>{ "Available Template Variables:" }</strong>
                        <ul>
                            <li><code>{ "{{speaker_name}}" }</code>{ " - Speaker's full name" }</li>
                            <li><code>{ "{{talk_title}}" }</code>{ " - Talk title" }</li>
                            <li><code>{ "{{talk_id}}" }</code>{ " - Talk ID" }</li>
                            <li><code>{ "{{reason}}" }</code>{ " - Optional reason (for rejections)" }</li>
                            <li><code>{ "{{schedule_date}}" }</code>{ " - Scheduled date" }</li>
                            <li><code>{ "{{schedule_time}}" }</code>{ " - Scheduled time" }</li>
                            <li><code>{ "{{track_name}}" }</code>{ " - Track/room name" }</li>
                        </ul>
                        <p><small>{ "Use " }<code>{ "{{#if variable}}...{{/if}}" }</code>{ " for conditional sections" }</small></p>
                    </div>

                    <form onsubmit={on_submit}>
                        <div class="form-group">
                            <label>{ "Template Name *" }</label>
                            <input
                                type="text"
                                value={(*name).clone()}
                                oninput={Callback::from({
                                    let name = name.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        name.set(input.value());
                                    }
                                })}
                                required=true
                            />
                        </div>

                        if !is_editing {
                            <div class="form-group">
                                <label>{ "Template Type *" }</label>
                                <select
                                    value={(*template_type).clone()}
                                    onchange={Callback::from({
                                        let template_type = template_type.clone();
                                        move |e: Event| {
                                            let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                            template_type.set(input.value());
                                        }
                                    })}
                                    required=true
                                >
                                    <option value="submission_confirmation">{ "Submission Confirmation" }</option>
                                    <option value="talk_pending">{ "Talk Pending" }</option>
                                    <option value="talk_accepted">{ "Talk Accepted" }</option>
                                    <option value="talk_rejected">{ "Talk Rejected" }</option>
                                    <option value="schedule_notification">{ "Schedule Notification" }</option>
                                    <option value="custom">{ "Custom" }</option>
                                </select>
                            </div>
                        }

                        <div class="form-group">
                            <label>{ "Email Subject *" }</label>
                            <input
                                type="text"
                                value={(*subject).clone()}
                                oninput={Callback::from({
                                    let subject = subject.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        subject.set(input.value());
                                    }
                                })}
                                required=true
                                placeholder="e.g. Your talk has been accepted: {{talk_title}}"
                            />
                        </div>

                        <div class="form-group">
                            <label>{ "Email Body *" }</label>
                            <textarea
                                value={(*body).clone()}
                                oninput={Callback::from({
                                    let body = body.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                        body.set(input.value());
                                    }
                                })}
                                required=true
                                rows="15"
                                placeholder="Dear {{speaker_name}},\n\nYour message here..."
                            />
                        </div>

                        <div class="form-group checkbox-group">
                            <label>
                                <input
                                    type="checkbox"
                                    checked={*is_default}
                                    onchange={Callback::from({
                                        let is_default = is_default.clone();
                                        move |e: Event| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            is_default.set(input.checked());
                                        }
                                    })}
                                />
                                { " Set as default template for this type" }
                            </label>
                        </div>

                        <button type="submit" class="btn-primary">
                            { if is_editing { "Update Template" } else { "Create Template" } }
                        </button>
                    </form>
                </div>
            }

            if *loading {
                <div class="loading">{ "Loading email templates..." }</div>
            } else if templates.is_empty() {
                <div class="empty-state">
                    <p>{ "No email templates configured yet." }</p>
                    <p>{ "Add your first template to customize emails sent to speakers." }</p>
                </div>
            } else {
                <div class="templates-list">
                    {
                        templates.iter().map(|template| {
                            let template_id = template.id.clone();
                            let template_for_edit = template.clone();
                            html! {
                                <div class="template-card" key={template.id.clone()}>
                                    <div class="template-header">
                                        <div>
                                            <h3>{ &template.name }</h3>
                                            <span class="template-type-badge">{ &template.template_type }</span>
                                            if template.is_default {
                                                <span class="default-badge">{ "Default" }</span>
                                            }
                                        </div>
                                        <div class="template-actions">
                                            <button
                                                onclick={create_edit_handler(template_for_edit)}
                                                class="btn-secondary btn-small"
                                            >
                                                { "Edit" }
                                            </button>
                                            <button
                                                onclick={create_delete_handler(template_id)}
                                                class="btn-danger btn-small"
                                            >
                                                { "Delete" }
                                            </button>
                                        </div>
                                    </div>
                                    <p class="template-subject"><strong>{ "Subject: " }</strong>{ &template.subject }</p>
                                    <p class="template-preview">{
                                        if template.body.len() > 150 {
                                            format!("{}...", &template.body[..150])
                                        } else {
                                            template.body.clone()
                                        }
                                    }</p>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            }
        </div>
    }
}
