use yew::prelude::*;
use std::collections::HashMap;
use crate::{
    services::{schedule_slots::ScheduleSlotService, tracks::TrackService, talks::TalkService},
    types::{ScheduleSlot, Track, Talk},
};

#[function_component(ScheduleBuilder)]
pub fn schedule_builder() -> Html {
    let slots = use_state(|| Vec::<ScheduleSlot>::new());
    let tracks = use_state(|| Vec::<Track>::new());
    let talks = use_state(|| Vec::<Talk>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let selected_date = use_state(|| None::<String>);

    // Fetch all data on mount
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
                        talks.set(talks_data);
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

    // Get unique dates from slots
    let dates: Vec<String> = {
        let mut unique_dates: Vec<String> = (*slots)
            .iter()
            .map(|s| s.slot_date.clone())
            .collect();
        unique_dates.sort();
        unique_dates.dedup();
        unique_dates
    };

    // Get filtered slots for selected date (or all if no date selected)
    let filtered_slots: Vec<ScheduleSlot> = match (*selected_date).as_ref() {
        Some(date) => (*slots).iter().filter(|s| &s.slot_date == date).cloned().collect(),
        None => (*slots).clone(),
    };

    // Group slots by time (start_time + end_time)
    let mut time_slots: Vec<(String, String)> = filtered_slots
        .iter()
        .map(|s| (s.start_time.clone(), s.end_time.clone()))
        .collect();
    time_slots.sort();
    time_slots.dedup();

    // Create a map of (track_id, time) -> slot
    let slot_map: HashMap<(String, String, String), ScheduleSlot> = filtered_slots
        .iter()
        .map(|s| {
            (
                (s.track_id.clone(), s.start_time.clone(), s.end_time.clone()),
                s.clone(),
            )
        })
        .collect();

    // Create a map of talk_id -> talk
    let talk_map: HashMap<String, Talk> = (*talks)
        .iter()
        .map(|t| (t.id.clone(), t.clone()))
        .collect();

    // Date selector handler
    let on_date_change = {
        let selected_date = selected_date.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            if value.is_empty() {
                selected_date.set(None);
            } else {
                selected_date.set(Some(value));
            }
        })
    };

    html! {
        <div class="schedule-builder-container">
            <div class="page-header">
                <h1>{ "Schedule Builder" }</h1>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if *loading {
                <div class="loading">{ "Loading schedule data..." }</div>
            } else if tracks.is_empty() {
                <div class="empty-state">
                    <p>{ "No tracks configured yet. Create tracks first." }</p>
                </div>
            } else if filtered_slots.is_empty() {
                <div class="empty-state">
                    <p>{ "No time slots configured yet. Create time slots first." }</p>
                </div>
            } else {
                <div class="schedule-controls">
                    <div class="date-filter">
                        <label for="date-select">{ "Filter by Date: " }</label>
                        <select id="date-select" onchange={on_date_change}>
                            <option value="">{ "All Dates" }</option>
                            {
                                dates.iter().map(|date| {
                                    html! {
                                        <option value={date.clone()} key={date.clone()}>
                                            { date }
                                        </option>
                                    }
                                }).collect::<Html>()
                            }
                        </select>
                    </div>
                </div>

                <div class="schedule-grid-container">
                    <table class="schedule-grid">
                        <thead>
                            <tr>
                                <th class="time-column">{ "Time" }</th>
                                {
                                    tracks.iter().map(|track| {
                                        html! {
                                            <th key={track.id.clone()} class="track-column">
                                                <div class="track-header">
                                                    <div class="track-name">{ &track.name }</div>
                                                    if let Some(capacity) = track.capacity {
                                                        <div class="track-capacity">{ format!("Capacity: {}", capacity) }</div>
                                                    }
                                                </div>
                                            </th>
                                        }
                                    }).collect::<Html>()
                                }
                            </tr>
                        </thead>
                        <tbody>
                            {
                                time_slots.iter().map(|(start_time, end_time)| {
                                    html! {
                                        <tr key={format!("{}-{}", start_time, end_time)}>
                                            <td class="time-cell">
                                                <div class="time-range">
                                                    { format!("{} - {}", start_time, end_time) }
                                                </div>
                                            </td>
                                            {
                                                tracks.iter().map(|track| {
                                                    let slot_key = (track.id.clone(), start_time.clone(), end_time.clone());
                                                    let slot_opt = slot_map.get(&slot_key);

                                                    match slot_opt {
                                                        Some(slot) => {
                                                            let talk_opt = slot.talk_id.as_ref().and_then(|tid| talk_map.get(tid));

                                                            html! {
                                                                <td key={format!("{}-{}-{}", track.id, start_time, end_time)} class="slot-cell">
                                                                    {
                                                                        if let Some(talk) = talk_opt {
                                                                            html! {
                                                                                <div class="assigned-talk-card">
                                                                                    <div class="talk-title">{ &talk.title }</div>
                                                                                    <div class="talk-speaker">{ &talk.speaker_name }</div>
                                                                                    <div class="talk-state">
                                                                                        <span class={format!("state-badge state-{:?}", talk.state).to_lowercase()}>
                                                                                            { format!("{:?}", talk.state) }
                                                                                        </span>
                                                                                    </div>
                                                                                </div>
                                                                            }
                                                                        } else {
                                                                            html! {
                                                                                <div class="empty-slot">
                                                                                    <span class="empty-label">{ "No talk assigned" }</span>
                                                                                </div>
                                                                            }
                                                                        }
                                                                    }
                                                                </td>
                                                            }
                                                        }
                                                        None => {
                                                            html! {
                                                                <td key={format!("{}-{}-{}", track.id, start_time, end_time)} class="slot-cell no-slot">
                                                                    <div class="no-slot-indicator">{ "—" }</div>
                                                                </td>
                                                            }
                                                        }
                                                    }
                                                }).collect::<Html>()
                                            }
                                        </tr>
                                    }
                                }).collect::<Html>()
                            }
                        </tbody>
                    </table>
                </div>

                <div class="schedule-legend">
                    <h3>{ "Legend" }</h3>
                    <div class="legend-items">
                        <div class="legend-item">
                            <span class="legend-color assigned"></span>
                            <span>{ "Assigned Talk" }</span>
                        </div>
                        <div class="legend-item">
                            <span class="legend-color empty"></span>
                            <span>{ "Empty Slot" }</span>
                        </div>
                        <div class="legend-item">
                            <span class="legend-color no-slot"></span>
                            <span>{ "No Slot Configured" }</span>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}
