use yew::prelude::*;
use std::collections::HashMap;
use crate::{
    services::schedule_slots::ScheduleSlotService,
    types::PublicScheduleSlot,
};

#[function_component(PublicSchedule)]
pub fn public_schedule() -> Html {
    let schedule = use_state(|| Vec::<PublicScheduleSlot>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let selected_date = use_state(|| None::<String>);

    // Fetch schedule on mount
    {
        let schedule = schedule.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match ScheduleSlotService::get_public_schedule().await {
                    Ok(slots) => {
                        schedule.set(slots);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load schedule: {}", e)));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Get unique dates from schedule
    let dates: Vec<String> = {
        let mut date_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for slot in schedule.iter() {
            date_set.insert(slot.slot_date.clone());
        }
        let mut dates: Vec<String> = date_set.into_iter().collect();
        dates.sort();
        dates
    };

    // Set default selected date to first date if not set
    if selected_date.is_none() && !dates.is_empty() {
        selected_date.set(Some(dates[0].clone()));
    }

    // Filter schedule by selected date
    let filtered_schedule: Vec<PublicScheduleSlot> = if let Some(date) = (*selected_date).as_ref() {
        schedule
            .iter()
            .filter(|slot| &slot.slot_date == date)
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    // Group schedule by track
    let schedule_by_track: HashMap<String, Vec<PublicScheduleSlot>> = {
        let mut map: HashMap<String, Vec<PublicScheduleSlot>> = HashMap::new();
        for slot in filtered_schedule.iter() {
            map.entry(slot.track_name.clone())
                .or_insert_with(Vec::new)
                .push(slot.clone());
        }
        map
    };

    // Get sorted track names
    let mut track_names: Vec<String> = schedule_by_track.keys().cloned().collect();
    track_names.sort();

    let date_selector = {
        let selected_date = selected_date.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            selected_date.set(Some(select.value()));
        })
    };

    html! {
        <div class="public-schedule-container">
            <div class="page-header">
                <h1>{ "Conference Schedule" }</h1>
            </div>

            if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            }

            if *loading {
                <div class="loading">{ "Loading schedule..." }</div>
            } else if dates.is_empty() {
                <div class="empty-state">
                    <p>{ "No schedule available yet. Check back later!" }</p>
                </div>
            } else {
                <>
                    <div class="date-selector">
                        <label>{ "Select Date: " }</label>
                        <select value={(*selected_date).clone().unwrap_or_default()} onchange={date_selector}>
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

                    <div class="schedule-grid">
                        {
                            track_names.iter().map(|track_name| {
                                let slots = schedule_by_track.get(track_name).unwrap();
                                html! {
                                    <div class="track-column" key={track_name.clone()}>
                                        <h2 class="track-header">{ track_name }</h2>
                                        <div class="track-slots">
                                            {
                                                slots.iter().map(|slot| {
                                                    html! {
                                                        <div class="schedule-slot" key={slot.id.clone()}>
                                                            <div class="slot-time">
                                                                { format!("{} - {}", &slot.start_time, &slot.end_time) }
                                                            </div>
                                                            {
                                                                if let Some(talk) = &slot.talk {
                                                                    html! {
                                                                        <div class="slot-talk">
                                                                            <h3 class="talk-title">{ &talk.title }</h3>
                                                                            <p class="talk-speaker">{ format!("Speaker: {}", &talk.speaker_name) }</p>
                                                                            <p class="talk-summary">{ &talk.short_summary }</p>
                                                                        </div>
                                                                    }
                                                                } else {
                                                                    html! {
                                                                        <div class="slot-empty">
                                                                            <p>{ "Available" }</p>
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
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </>
            }
        </div>
    }
}
