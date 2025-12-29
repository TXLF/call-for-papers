use yew::prelude::*;
use crate::{
    services::{schedule_slots::ScheduleSlotService, tracks::TrackService},
    types::{ScheduleSlot, Track, CreateScheduleSlotRequest},
};

#[function_component(ManageScheduleSlots)]
pub fn manage_schedule_slots() -> Html {
    let slots = use_state(|| Vec::<ScheduleSlot>::new());
    let tracks = use_state(|| Vec::<Track>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_create_form = use_state(|| false);

    // Form state
    let track_id = use_state(|| String::new());
    let slot_date = use_state(|| String::new());
    let start_time = use_state(|| String::new());
    let end_time = use_state(|| String::new());

    // Fetch slots and tracks on mount
    {
        let slots = slots.clone();
        let tracks = tracks.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);

                // Fetch both slots and tracks in parallel
                let slots_result = ScheduleSlotService::list_schedule_slots().await;
                let tracks_result = TrackService::list_tracks().await;

                match (slots_result, tracks_result) {
                    (Ok(slots_data), Ok(tracks_data)) => {
                        slots.set(slots_data);
                        tracks.set(tracks_data);
                        error.set(None);
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Create slot handler
    let on_create = {
        let slots = slots.clone();
        let track_id = track_id.clone();
        let slot_date = slot_date.clone();
        let start_time = start_time.clone();
        let end_time = end_time.clone();
        let show_create_form = show_create_form.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let slots = slots.clone();
            let track_id_val = (*track_id).clone();
            let date_val = (*slot_date).clone();
            let start_val = (*start_time).clone();
            let end_val = (*end_time).clone();
            let show_create_form = show_create_form.clone();
            let error = error.clone();
            let track_id = track_id.clone();
            let slot_date = slot_date.clone();
            let start_time = start_time.clone();
            let end_time = end_time.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Using dummy conference_id - in real impl, this would come from config or selection
                let dummy_conf_id = "00000000-0000-0000-0000-000000000000".to_string();
                let request = CreateScheduleSlotRequest {
                    conference_id: dummy_conf_id,
                    track_id: track_id_val,
                    slot_date: date_val,
                    start_time: start_val,
                    end_time: end_val,
                };

                match ScheduleSlotService::create_schedule_slot(request).await {
                    Ok(new_slot) => {
                        let mut current_slots = (*slots).clone();
                        current_slots.push(new_slot);
                        slots.set(current_slots);
                        track_id.set(String::new());
                        slot_date.set(String::new());
                        start_time.set(String::new());
                        end_time.set(String::new());
                        show_create_form.set(false);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create slot: {}", e)));
                    }
                }
            });
        })
    };

    // Delete slot handler
    let create_delete_handler = |slot_id: String| {
        let slots = slots.clone();
        let error = error.clone();

        Callback::from(move |_: MouseEvent| {
            let slots = slots.clone();
            let slot_id = slot_id.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match ScheduleSlotService::delete_schedule_slot(&slot_id).await {
                    Ok(_) => {
                        let mut current_slots = (*slots).clone();
                        current_slots.retain(|s| s.id != slot_id);
                        slots.set(current_slots);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to delete slot: {}", e)));
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
        <div class="manage-schedule-slots-container">
            <div class="page-header">
                <h1>{ "Manage Time Slots" }</h1>
                <button
                    onclick={toggle_form}
                    class="btn-primary"
                >
                    { if show_create_form_display { "Cancel" } else { "Add Time Slot" } }
                </button>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if *show_create_form {
                <div class="create-form-card">
                    <h2>{ "Create New Time Slot" }</h2>
                    <form onsubmit={on_create}>
                        <div class="form-group">
                            <label>{ "Track/Room *" }</label>
                            <select
                                value={(*track_id).clone()}
                                onchange={Callback::from({
                                    let track_id = track_id.clone();
                                    move |e: Event| {
                                        let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                        track_id.set(select.value());
                                    }
                                })}
                                required=true
                            >
                                <option value="">{ "Select a track" }</option>
                                {
                                    tracks.iter().map(|t| {
                                        html! {
                                            <option value={t.id.clone()} key={t.id.clone()}>
                                                { &t.name }
                                            </option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                        </div>

                        <div class="form-group">
                            <label>{ "Date *" }</label>
                            <input
                                type="date"
                                value={(*slot_date).clone()}
                                oninput={Callback::from({
                                    let slot_date = slot_date.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        slot_date.set(input.value());
                                    }
                                })}
                                required=true
                            />
                        </div>

                        <div class="form-group">
                            <label>{ "Start Time *" }</label>
                            <input
                                type="time"
                                value={(*start_time).clone()}
                                oninput={Callback::from({
                                    let start_time = start_time.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        start_time.set(input.value());
                                    }
                                })}
                                required=true
                            />
                        </div>

                        <div class="form-group">
                            <label>{ "End Time *" }</label>
                            <input
                                type="time"
                                value={(*end_time).clone()}
                                oninput={Callback::from({
                                    let end_time = end_time.clone();
                                    move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        end_time.set(input.value());
                                    }
                                })}
                                required=true
                            />
                        </div>

                        <button type="submit" class="btn-primary">{ "Create Time Slot" }</button>
                    </form>
                </div>
            }

            if *loading {
                <div class="loading">{ "Loading time slots..." }</div>
            } else if slots.is_empty() {
                <div class="empty-state">
                    <p>{ "No time slots configured yet." }</p>
                </div>
            } else {
                <div class="slots-list">
                    {
                        slots.iter().map(|slot| {
                            let slot_id = slot.id.clone();
                            let track_name = tracks.iter()
                                .find(|t| t.id == slot.track_id)
                                .map(|t| t.name.as_str())
                                .unwrap_or("Unknown Track");

                            html! {
                                <div class="slot-card" key={slot.id.clone()}>
                                    <div class="slot-header">
                                        <h3>{ track_name }</h3>
                                        <button
                                            onclick={create_delete_handler(slot_id)}
                                            class="btn-danger btn-small"
                                        >
                                            { "Delete" }
                                        </button>
                                    </div>
                                    <div class="slot-details">
                                        <p><strong>{ "Date:" }</strong> { &slot.slot_date }</p>
                                        <p><strong>{ "Time:" }</strong> { format!("{} - {}", &slot.start_time, &slot.end_time) }</p>
                                        {
                                            if let Some(talk_id) = &slot.talk_id {
                                                html! { <p><strong>{ "Assigned Talk:" }</strong> { talk_id }</p> }
                                            } else {
                                                html! { <p class="unassigned">{ "No talk assigned" }</p> }
                                            }
                                        }
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            }
        </div>
    }
}
