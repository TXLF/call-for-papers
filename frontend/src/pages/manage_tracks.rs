use yew::prelude::*;
use crate::{
    services::tracks::TrackService,
    types::{Track, CreateTrackRequest, UpdateTrackRequest},
};

#[function_component(ManageTracks)]
pub fn manage_tracks() -> Html {
    let tracks = use_state(|| Vec::<Track>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_create_form = use_state(|| false);
    let _editing_track_id = use_state(|| None::<String>);

    // Form state
    let name = use_state(|| String::new());
    let description = use_state(|| String::new());
    let capacity = use_state(|| String::new());

    // Fetch tracks on mount
    {
        let tracks = tracks.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
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

    // Create track handler
    let on_create = {
        let tracks = tracks.clone();
        let name = name.clone();
        let description = description.clone();
        let capacity = capacity.clone();
        let show_create_form = show_create_form.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let tracks = tracks.clone();
            let name_val = (*name).clone();
            let desc_val = if (*description).trim().is_empty() { None } else { Some((*description).clone()) };
            let cap_val = (*capacity).parse::<i32>().ok();
            let show_create_form = show_create_form.clone();
            let error = error.clone();
            let name = name.clone();
            let description = description.clone();
            let capacity = capacity.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Using dummy conference_id - in real impl, this would come from config or selection
                let dummy_conf_id = "00000000-0000-0000-0000-000000000000".to_string();
                let request = CreateTrackRequest {
                    conference_id: dummy_conf_id,
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
                        show_create_form.set(false);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create track: {}", e)));
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

    let show_create_form_display = *show_create_form;
    let toggle_form = {
        let show_create_form = show_create_form.clone();
        Callback::from(move |_| show_create_form.set(!*show_create_form))
    };

    html! {
        <div class="manage-tracks-container">
            <div class="page-header">
                <h1>{ "Manage Tracks" }</h1>
                <button
                    onclick={toggle_form}
                    class="btn-primary"
                >
                    { if show_create_form_display { "Cancel" } else { "Add Track" } }
                </button>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if *show_create_form {
                <div class="create-form-card">
                    <h2>{ "Create New Track" }</h2>
                    <form onsubmit={on_create}>
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

                        <button type="submit" class="btn-primary">{ "Create Track" }</button>
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
                            html! {
                                <div class="track-card" key={track.id.clone()}>
                                    <div class="track-header">
                                        <h3>{ &track.name }</h3>
                                        <button
                                            onclick={create_delete_handler(track_id)}
                                            class="btn-danger btn-small"
                                        >
                                            { "Delete" }
                                        </button>
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
