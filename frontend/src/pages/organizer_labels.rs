use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::{
    services::labels::LabelService,
    types::{Label, CreateLabelRequest, UpdateLabelRequest},
};

#[function_component(OrganizerLabels)]
pub fn organizer_labels() -> Html {
    let labels = use_state(|| Vec::<Label>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let search_term = use_state(|| String::new());
    let editing_label = use_state(|| None::<Label>);
    let show_create_form = use_state(|| false);

    // Form states for create
    let create_name = use_state(|| String::new());
    let create_description = use_state(|| String::new());
    let create_color = use_state(|| String::from("#3498db"));

    // Form states for edit
    let edit_name = use_state(|| String::new());
    let edit_description = use_state(|| String::new());
    let edit_color = use_state(|| String::new());

    // Fetch labels on mount
    {
        let labels = labels.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match LabelService::list_labels().await {
                    Ok(data) => {
                        labels.set(data);
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

    // Create label handler
    let on_create_submit = {
        let labels = labels.clone();
        let create_name = create_name.clone();
        let create_description = create_description.clone();
        let create_color = create_color.clone();
        let show_create_form = show_create_form.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let labels = labels.clone();
            let name = (*create_name).clone();
            let description = if create_description.is_empty() {
                None
            } else {
                Some((*create_description).clone())
            };
            let color = if create_color.is_empty() {
                None
            } else {
                Some((*create_color).clone())
            };
            let create_name = create_name.clone();
            let create_description = create_description.clone();
            let create_color = create_color.clone();
            let show_create_form = show_create_form.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let request = CreateLabelRequest { name, description, color };
                match LabelService::create_label(request).await {
                    Ok(new_label) => {
                        let mut current_labels = (*labels).clone();
                        current_labels.push(new_label);
                        labels.set(current_labels);
                        create_name.set(String::new());
                        create_description.set(String::new());
                        create_color.set(String::from("#3498db"));
                        show_create_form.set(false);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create label: {}", e)));
                    }
                }
            });
        })
    };

    // Edit label handler
    let on_edit_submit = {
        let labels = labels.clone();
        let editing_label = editing_label.clone();
        let edit_name = edit_name.clone();
        let edit_description = edit_description.clone();
        let edit_color = edit_color.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if let Some(label) = (*editing_label).clone() {
                let labels = labels.clone();
                let editing_label = editing_label.clone();
                let error = error.clone();

                let name = if !edit_name.is_empty() {
                    Some((*edit_name).clone())
                } else {
                    None
                };
                let description = if !edit_description.is_empty() {
                    Some((*edit_description).clone())
                } else {
                    None
                };
                let color = if !edit_color.is_empty() {
                    Some((*edit_color).clone())
                } else {
                    None
                };

                wasm_bindgen_futures::spawn_local(async move {
                    let request = UpdateLabelRequest { name, description, color };
                    match LabelService::update_label(&label.id, request).await {
                        Ok(updated_label) => {
                            let mut current_labels = (*labels).clone();
                            if let Some(idx) = current_labels.iter().position(|l| l.id == updated_label.id) {
                                current_labels[idx] = updated_label;
                            }
                            labels.set(current_labels);
                            editing_label.set(None);
                            error.set(None);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to update label: {}", e)));
                        }
                    }
                });
            }
        })
    };

    // Delete label handler
    let on_delete_click = {
        let labels = labels.clone();
        let error = error.clone();

        Callback::from(move |label_id: String| {
            if !web_sys::window()
                .unwrap()
                .confirm_with_message("Are you sure you want to delete this label? It will be removed from all talks.")
                .unwrap_or(false)
            {
                return;
            }

            let labels = labels.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match LabelService::delete_label(&label_id).await {
                    Ok(()) => {
                        let current_labels = (*labels).clone();
                        let updated_labels: Vec<Label> = current_labels
                            .into_iter()
                            .filter(|l| l.id != label_id)
                            .collect();
                        labels.set(updated_labels);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to delete label: {}", e)));
                    }
                }
            });
        })
    };

    // Start editing a label
    let on_edit_click = {
        let editing_label = editing_label.clone();
        let edit_name = edit_name.clone();
        let edit_description = edit_description.clone();
        let edit_color = edit_color.clone();

        Callback::from(move |label: Label| {
            edit_name.set(label.name.clone());
            edit_description.set(label.description.clone().unwrap_or_default());
            edit_color.set(label.color.clone().unwrap_or_default());
            editing_label.set(Some(label));
        })
    };

    // Cancel editing
    let on_cancel_edit = {
        let editing_label = editing_label.clone();
        Callback::from(move |_| {
            editing_label.set(None);
        })
    };

    // Filter labels based on search term
    let filtered_labels: Vec<Label> = (*labels)
        .iter()
        .filter(|label| {
            let term = search_term.to_lowercase();
            label.name.to_lowercase().contains(&term) ||
            label.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&term))
        })
        .cloned()
        .collect();

    let show_create_form_clone = show_create_form.clone();
    let create_color_clone = create_color.clone();

    html! {
        <div class="organizer-labels">
            <div class="labels-header">
                <h1>{ "Label Management" }</h1>
                <button
                    class="btn btn-primary"
                    onclick={Callback::from(move |_| show_create_form_clone.set(!*show_create_form_clone))}
                >
                    { if *show_create_form { "Cancel" } else { "+ Create Label" } }
                </button>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            // Create form
            if *show_create_form {
                <div class="label-form-card">
                    <h3>{ "Create New Label" }</h3>
                    <form onsubmit={on_create_submit}>
                        <div class="form-group">
                            <label for="create-name">{ "Name *" }</label>
                            <input
                                type="text"
                                id="create-name"
                                required=true
                                value={(*create_name).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    create_name.set(input.value());
                                })}
                            />
                        </div>

                        <div class="form-group">
                            <label for="create-description">{ "Description" }</label>
                            <textarea
                                id="create-description"
                                rows="2"
                                value={(*create_description).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    create_description.set(input.value());
                                })}
                            />
                        </div>

                        <div class="form-group">
                            <label for="create-color">{ "Color" }</label>
                            <div class="color-picker-group">
                                <input
                                    type="color"
                                    id="create-color"
                                    value={(*create_color).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        create_color_clone.set(input.value());
                                    })}
                                />
                                <span class="color-preview" style={format!("background-color: {}", *create_color)}>
                                    { &*create_color }
                                </span>
                            </div>
                        </div>

                        <div class="form-actions">
                            <button type="submit" class="btn btn-primary">{ "Create Label" }</button>
                        </div>
                    </form>
                </div>
            }

            // Search bar
            <div class="search-bar">
                <input
                    type="text"
                    placeholder="Search labels..."
                    value={(*search_term).clone()}
                    oninput={Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        search_term.set(input.value());
                    })}
                />
            </div>

            // Labels list
            if *loading {
                <div class="loading">{ "Loading labels..." }</div>
            } else if filtered_labels.is_empty() {
                <div class="no-data">
                    { if labels.is_empty() {
                        "No labels yet. Create one to get started!"
                    } else {
                        "No labels match your search."
                    }}
                </div>
            } else {
                <div class="labels-grid">
                    {
                        filtered_labels.iter().map(|label| {
                            let label_id = label.id.clone();
                            let label_for_edit = label.clone();
                            let label_for_delete = label.id.clone();
                            let on_edit = on_edit_click.clone();
                            let on_delete = on_delete_click.clone();
                            let is_editing = editing_label.as_ref().map_or(false, |l| l.id == label.id);

                            // Clone edit state handles for use in closures
                            let edit_name_clone = edit_name.clone();
                            let edit_description_clone = edit_description.clone();
                            let edit_color_clone = edit_color.clone();
                            let edit_color_clone2 = edit_color.clone();

                            html! {
                                <div class="label-card" key={label_id}>
                                    if is_editing {
                                        // Edit form
                                        <form onsubmit={on_edit_submit.clone()} class="label-edit-form">
                                            <div class="form-group">
                                                <input
                                                    type="text"
                                                    placeholder="Label name"
                                                    value={(*edit_name).clone()}
                                                    oninput={Callback::from(move |e: InputEvent| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        edit_name_clone.set(input.value());
                                                    })}
                                                />
                                            </div>
                                            <div class="form-group">
                                                <textarea
                                                    placeholder="Description"
                                                    rows="2"
                                                    value={(*edit_description).clone()}
                                                    oninput={Callback::from(move |e: InputEvent| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        edit_description_clone.set(input.value());
                                                    })}
                                                />
                                            </div>
                                            <div class="form-group">
                                                <div class="color-picker-group">
                                                    <input
                                                        type="color"
                                                        value={(*edit_color).clone()}
                                                        oninput={Callback::from(move |e: InputEvent| {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            edit_color_clone.set(input.value());
                                                        })}
                                                    />
                                                    <span class="color-preview" style={format!("background-color: {}", *edit_color_clone2)}>
                                                        { &*edit_color_clone2 }
                                                    </span>
                                                </div>
                                            </div>
                                            <div class="form-actions">
                                                <button type="submit" class="btn btn-sm btn-primary">{ "Save" }</button>
                                                <button
                                                    type="button"
                                                    class="btn btn-sm btn-secondary"
                                                    onclick={on_cancel_edit.clone()}
                                                >
                                                    { "Cancel" }
                                                </button>
                                            </div>
                                        </form>
                                    } else {
                                        // Display mode
                                        <div class="label-display">
                                            <div class="label-header">
                                                <div
                                                    class="label-badge"
                                                    style={format!(
                                                        "background-color: {}",
                                                        label.color.as_ref().unwrap_or(&"#95a5a6".to_string())
                                                    )}
                                                >
                                                    { &label.name }
                                                </div>
                                                if label.is_ai_generated {
                                                    <span class="ai-badge" title="AI Generated">{ "🤖 AI" }</span>
                                                }
                                            </div>

                                            if let Some(desc) = &label.description {
                                                <p class="label-description">{ desc }</p>
                                            }

                                            <div class="label-meta">
                                                <span class="label-date">
                                                    { format!("Created: {}", &label.created_at[..10]) }
                                                </span>
                                            </div>

                                            <div class="label-actions">
                                                <button
                                                    class="btn btn-sm btn-secondary"
                                                    onclick={Callback::from(move |_| on_edit.emit(label_for_edit.clone()))}
                                                >
                                                    { "Edit" }
                                                </button>
                                                <button
                                                    class="btn btn-sm btn-danger"
                                                    onclick={Callback::from(move |_| on_delete.emit(label_for_delete.clone()))}
                                                >
                                                    { "Delete" }
                                                </button>
                                            </div>
                                        </div>
                                    }
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            }
        </div>
    }
}
