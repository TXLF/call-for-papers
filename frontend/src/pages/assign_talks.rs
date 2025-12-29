use yew::prelude::*;
use crate::{
    services::{schedule_slots::ScheduleSlotService, tracks::TrackService, talks::TalkService},
    types::{ScheduleSlot, Track, Talk, TalkState, AssignTalkRequest},
};

#[function_component(AssignTalks)]
pub fn assign_talks() -> Html {
    let slots = use_state(|| Vec::<ScheduleSlot>::new());
    let tracks = use_state(|| Vec::<Track>::new());
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let assigning_slot_id = use_state(|| None::<String>);

    // Fetch slots, tracks, and accepted talks on mount
    {
        let slots = slots.clone();
        let tracks = tracks.clone();
        let talks = talks.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);

                // Fetch all data in parallel
                let slots_result = ScheduleSlotService::list_schedule_slots().await;
                let tracks_result = TrackService::list_tracks().await;
                let talks_result = TalkService::list_all_talks(None).await;

                match (slots_result, tracks_result, talks_result) {
                    (Ok(slots_data), Ok(tracks_data), Ok(talks_data)) => {
                        slots.set(slots_data);
                        tracks.set(tracks_data);
                        // Filter to only accepted talks
                        let accepted_talks: Vec<Talk> = talks_data
                            .into_iter()
                            .filter(|t| matches!(t.state, TalkState::Accepted))
                            .collect();
                        talks.set(accepted_talks);
                        error.set(None);
                    }
                    (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Assign talk handler
    let create_assign_handler = |slot_id: String, talk_id: String| {
        let slots = slots.clone();
        let error = error.clone();
        let assigning_slot_id = assigning_slot_id.clone();

        Callback::from(move |_: MouseEvent| {
            let slots = slots.clone();
            let slot_id = slot_id.clone();
            let talk_id = talk_id.clone();
            let error = error.clone();
            let assigning_slot_id = assigning_slot_id.clone();

            assigning_slot_id.set(Some(slot_id.clone()));
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let request = AssignTalkRequest { talk_id };
                match ScheduleSlotService::assign_talk_to_slot(&slot_id, request).await {
                    Ok(updated_slot) => {
                        let mut current_slots = (*slots).clone();
                        if let Some(slot) = current_slots.iter_mut().find(|s| s.id == slot_id) {
                            *slot = updated_slot;
                        }
                        slots.set(current_slots);
                        assigning_slot_id.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to assign talk: {}", e)));
                        assigning_slot_id.set(None);
                    }
                }
            });
        })
    };

    // Unassign talk handler
    let create_unassign_handler = |slot_id: String| {
        let slots = slots.clone();
        let error = error.clone();
        let assigning_slot_id = assigning_slot_id.clone();

        Callback::from(move |_: MouseEvent| {
            let slots = slots.clone();
            let slot_id = slot_id.clone();
            let error = error.clone();
            let assigning_slot_id = assigning_slot_id.clone();

            assigning_slot_id.set(Some(slot_id.clone()));
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match ScheduleSlotService::unassign_talk_from_slot(&slot_id).await {
                    Ok(updated_slot) => {
                        let mut current_slots = (*slots).clone();
                        if let Some(slot) = current_slots.iter_mut().find(|s| s.id == slot_id) {
                            *slot = updated_slot;
                        }
                        slots.set(current_slots);
                        assigning_slot_id.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to unassign talk: {}", e)));
                        assigning_slot_id.set(None);
                    }
                }
            });
        })
    };

    html! {
        <div class="assign-talks-container">
            <div class="page-header">
                <h1>{ "Assign Talks to Schedule" }</h1>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if *loading {
                <div class="loading">{ "Loading schedule and talks..." }</div>
            } else if slots.is_empty() {
                <div class="empty-state">
                    <p>{ "No time slots configured yet. Create time slots first." }</p>
                </div>
            } else {
                <div class="assign-layout">
                    <div class="schedule-slots-section">
                        <h2>{ "Schedule Slots" }</h2>
                        <div class="slots-grid">
                            {
                                slots.iter().map(|slot| {
                                    let slot_id = slot.id.clone();
                                    let track_name = tracks.iter()
                                        .find(|t| t.id == slot.track_id)
                                        .map(|t| t.name.as_str())
                                        .unwrap_or("Unknown Track");

                                    let is_assigning = (*assigning_slot_id).as_ref() == Some(&slot.id);

                                    let assigned_talk = slot.talk_id.as_ref().and_then(|tid| {
                                        talks.iter().find(|t| &t.id == tid)
                                    });

                                    html! {
                                        <div class="slot-assignment-card" key={slot.id.clone()}>
                                            <div class="slot-info">
                                                <h3>{ track_name }</h3>
                                                <p class="slot-time">
                                                    { format!("{} | {} - {}", &slot.slot_date, &slot.start_time, &slot.end_time) }
                                                </p>
                                            </div>

                                            {
                                                if let Some(talk) = assigned_talk {
                                                    html! {
                                                        <div class="assigned-talk">
                                                            <div class="talk-info">
                                                                <strong>{ "Assigned:" }</strong>
                                                                <p class="talk-title">{ &talk.title }</p>
                                                                <p class="talk-speaker">{ format!("Speaker: {}", &talk.speaker_name) }</p>
                                                            </div>
                                                            <button
                                                                onclick={create_unassign_handler(slot_id.clone())}
                                                                disabled={is_assigning}
                                                                class="btn-danger btn-small"
                                                            >
                                                                { if is_assigning { "Unassigning..." } else { "Unassign" } }
                                                            </button>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {
                                                        <div class="unassigned-slot">
                                                            <p class="unassigned-label">{ "No talk assigned" }</p>
                                                            <div class="assign-dropdown">
                                                                <select
                                                                    class="talk-select"
                                                                    onchange={Callback::from({
                                                                        let slots = slots.clone();
                                                                        let error = error.clone();
                                                                        let assigning_slot_id = assigning_slot_id.clone();
                                                                        let slot_id = slot_id.clone();
                                                                        move |e: Event| {
                                                                            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                                            let talk_id = select.value();
                                                                            if !talk_id.is_empty() {
                                                                                let slots = slots.clone();
                                                                                let error = error.clone();
                                                                                let assigning_slot_id = assigning_slot_id.clone();
                                                                                let slot_id = slot_id.clone();

                                                                                assigning_slot_id.set(Some(slot_id.clone()));
                                                                                error.set(None);

                                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                                    let request = AssignTalkRequest { talk_id };
                                                                                    match ScheduleSlotService::assign_talk_to_slot(&slot_id, request).await {
                                                                                        Ok(updated_slot) => {
                                                                                            let mut current_slots = (*slots).clone();
                                                                                            if let Some(slot) = current_slots.iter_mut().find(|s| s.id == slot_id) {
                                                                                                *slot = updated_slot;
                                                                                            }
                                                                                            slots.set(current_slots);
                                                                                            assigning_slot_id.set(None);
                                                                                        }
                                                                                        Err(e) => {
                                                                                            error.set(Some(format!("Failed to assign talk: {}", e)));
                                                                                            assigning_slot_id.set(None);
                                                                                        }
                                                                                    }
                                                                                });
                                                                                select.set_value("");
                                                                            }
                                                                        }
                                                                    })}
                                                                    disabled={is_assigning}
                                                                >
                                                                    <option value="">{ "Select a talk to assign..." }</option>
                                                                    {
                                                                        talks.iter()
                                                                            .filter(|t| !slots.iter().any(|s| s.talk_id.as_ref() == Some(&t.id)))
                                                                            .map(|talk| {
                                                                                html! {
                                                                                    <option value={talk.id.clone()} key={talk.id.clone()}>
                                                                                        { format!("{} - {}", &talk.title, &talk.speaker_name) }
                                                                                    </option>
                                                                                }
                                                                            }).collect::<Html>()
                                                                    }
                                                                </select>
                                                            </div>
                                                        </div>
                                                    }
                                                }
                                            }
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    </div>

                    <div class="available-talks-section">
                        <h2>{ format!("Available Accepted Talks ({})", talks.iter().filter(|t| !slots.iter().any(|s| s.talk_id.as_ref() == Some(&t.id))).count()) }</h2>
                        <div class="talks-list">
                            {
                                talks.iter()
                                    .filter(|t| !slots.iter().any(|s| s.talk_id.as_ref() == Some(&t.id)))
                                    .map(|talk| {
                                        html! {
                                            <div class="available-talk-card" key={talk.id.clone()}>
                                                <h4>{ &talk.title }</h4>
                                                <p class="talk-speaker">{ format!("Speaker: {}", &talk.speaker_name) }</p>
                                                <p class="talk-summary">{ &talk.short_summary }</p>
                                            </div>
                                        }
                                    }).collect::<Html>()
                            }
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}
