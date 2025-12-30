use yew::prelude::*;
use crate::{
    services::{tracks::TrackService, conferences::ConferenceService},
    types::{Track, CreateTrackRequest, UpdateTrackRequest},
};

#[function_component(ManageTracks)]
pub fn manage_tracks() -> Html {
    let tracks = use_state(|| Vec::<Track>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let editing_track = use_state(|| None::<Track>);
    let conference_id = use_state(|| None::<String>);

    // Form state
    let name = use_state(|| String::new());
    let description = use_state(|| String::new());
    let capacity = use_state(|| String::new());

    // Fetch active conference and tracks on mount
    {
        let conference_id = conference_id.clone();
        let tracks = tracks.clone();
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

                // Fetch tracks
                match TrackService::list_tracks().await {
                    Ok(data) => {
                        tracks.set(data);
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
        let tracks = tracks.clone();
        let conference_id = conference_id.clone();
        let editing_track = editing_track.clone();
        let name = name.clone();
        let description = description.clone();
        let capacity = capacity.clone();
        let show_form = show_form.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let tracks = tracks.clone();
            let conf_id = (*conference_id).clone();
            let editing = (*editing_track).clone();
            let name_val = (*name).clone();
            let desc_val = if (*description).trim().is_empty() { None } else { Some((*description).clone()) };
            let cap_val = (*capacity).parse::<i32>().ok();
            let show_form = show_form.clone();
            let editing_track = editing_track.clone();
            let error = error.clone();
            let name = name.clone();
            let description = description.clone();
            let capacity = capacity.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Some(track) = editing {
                    // Update existing track
                    let request = UpdateTrackRequest {
                        name: Some(name_val),
                        description: desc_val,
                        capacity: cap_val,
                    };

                    match TrackService::update_track(&track.id, request).await {
                        Ok(updated_track) => {
                            let mut current_tracks = (*tracks).clone();
                            if let Some(index) = current_tracks.iter().position(|t| t.id == track.id) {
                                current_tracks[index] = updated_track;
                            }
                            tracks.set(current_tracks);
                            name.set(String::new());
                            description.set(String::new());
                            capacity.set(String::new());
                            show_form.set(false);
                            editing_track.set(None);
                            error.set(None);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to update track: {}", e)));
                        }
                    }
                } else {
                    // Create new track
                    let conf_id = match conf_id {
                        Some(id) => id,
                        None => {
                            error.set(Some("No active conference found".to_string()));
                            return;
                        }
                    };

                    let request = CreateTrackRequest {
                        conference_id: conf_id,
                        name: name_val,
                        description: desc_val,
                        capacity: cap_val,
                    };

                    match TrackService::create_track(request).await {
                        Ok(new_track) => {
                            let mut current_tracks = (*tracks).clone();
                            current_tracks.push(new_track);
                            tracks.set(current_tracks);
                            name.set(String::new());
                            description.set(String::new());
                            capacity.set(String::new());
                            show_form.set(false);
                            error.set(None);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to create track: {}", e)));
                        }
                    }
                }
            });
        })
    };

    // Delete track handler
    let create_delete_handler = |track_id: String| {
        let tracks = tracks.clone();
        let error = error.clone();

        Callback::from(move |_: MouseEvent| {
            let tracks = tracks.clone();
            let track_id = track_id.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match TrackService::delete_track(&track_id).await {
                    Ok(_) => {
                        let mut current_tracks = (*tracks).clone();
                        current_tracks.retain(|t| t.id != track_id);
                        tracks.set(current_tracks);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to delete track: {}", e)));
                    }
                }
            });
        })
    };

    // Edit button handler
    let create_edit_handler = |track: Track| {
        let show_form = show_form.clone();
        let editing_track = editing_track.clone();
        let name = name.clone();
        let description = description.clone();
        let capacity = capacity.clone();

        Callback::from(move |_: MouseEvent| {
            name.set(track.name.clone());
            description.set(track.description.clone().unwrap_or_default());
            capacity.set(track.capacity.map(|c| c.to_string()).unwrap_or_default());
            editing_track.set(Some(track.clone()));
            show_form.set(true);
        })
    };

    let show_form_display = *show_form;
    let is_editing = editing_track.is_some();

    let toggle_form = {
        let show_form = show_form.clone();
        let editing_track = editing_track.clone();
        let name = name.clone();
        let description = description.clone();
        let capacity = capacity.clone();

        Callback::from(move |_| {
            if *show_form {
                // Cancel - clear form
                name.set(String::new());
                description.set(String::new());
                capacity.set(String::new());
                editing_track.set(None);
                show_form.set(false);
            } else {
                // Open create form
                name.set(String::new());
                description.set(String::new());
                capacity.set(String::new());
                editing_track.set(None);
                show_form.set(true);
            }
        })
    };

    html! {
        <div class="manage-tracks-container">
            <div class="page-header">
                <h1>{ "Manage Tracks" }</h1>
                <button
                    onclick={toggle_form}
                    class="btn-primary"
                >
                    { if show_form_display { "Cancel" } else { "Add Track" } }
                </button>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if *show_form {
                <div class="create-form-card">
                    <h2>{ if is_editing { "Edit Track" } else { "Create New Track" } }</h2>
                    <form onsubmit={on_submit}>
                        <div class="form-group">
                            <label>{ "Track Name *" }</label>
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

                        <div class="form-group">
                            <label>{ "Description" }</label>
                            <textarea
                                value={(*description).clone()}
                                oninput={Callback::from({
                                    let description = description.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                        description.set(input.value());
                                    }
                                })}
                            />
                        </div>

                        <div class="form-group">
                            <label>{ "Capacity" }</label>
                            <input
                                type="number"
                                value={(*capacity).clone()}
                                oninput={Callback::from({
                                    let capacity = capacity.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        capacity.set(input.value());
                                    }
                                })}
                                min="1"
                            />
                        </div>

                        <button type="submit" class="btn-primary">
                            { if is_editing { "Update Track" } else { "Create Track" } }
                        </button>
                    </form>
                </div>
            }

            if *loading {
                <div class="loading">{ "Loading tracks..." }</div>
            } else if tracks.is_empty() {
                <div class="empty-state">
                    <p>{ "No tracks configured yet." }</p>
                </div>
            } else {
                <div class="tracks-list">
                    {
                        tracks.iter().map(|track| {
                            let track_id = track.id.clone();
                            let track_for_edit = track.clone();
                            html! {
                                <div class="track-card" key={track.id.clone()}>
                                    <div class="track-header">
                                        <h3>{ &track.name }</h3>
                                        <div class="track-actions">
                                            <button
                                                onclick={create_edit_handler(track_for_edit)}
                                                class="btn-secondary btn-small"
                                            >
                                                { "Edit" }
                                            </button>
                                            <button
                                                onclick={create_delete_handler(track_id)}
                                                class="btn-danger btn-small"
                                            >
                                                { "Delete" }
                                            </button>
                                        </div>
                                    </div>
                                    if let Some(desc) = &track.description {
                                        <p class="track-description">{ desc }</p>
                                    }
                                    if let Some(cap) = track.capacity {
                                        <p class="track-capacity">{ format!("Capacity: {}", cap) }</p>
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
