use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;

use crate::{
    components::rating_stars::RatingStars,
    services::ratings::RatingService,
    types::Rating,
};

#[derive(Properties, PartialEq)]
pub struct RatingFormProps {
    /// ID of the talk being rated
    pub talk_id: String,
    /// Existing rating if user has already rated this talk
    #[prop_or_default]
    pub existing_rating: Option<Rating>,
    /// Callback when rating is successfully submitted
    #[prop_or_default]
    pub on_success: Option<Callback<Rating>>,
    /// Callback when rating is deleted
    #[prop_or_default]
    pub on_delete: Option<Callback<()>>,
}

#[function_component(RatingForm)]
pub fn rating_form(props: &RatingFormProps) -> Html {
    let rating = use_state(|| {
        props
            .existing_rating
            .as_ref()
            .map(|r| r.rating)
            .unwrap_or(0)
    });
    let notes = use_state(|| {
        props
            .existing_rating
            .as_ref()
            .and_then(|r| r.notes.clone())
            .unwrap_or_default()
    });
    let is_loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success_message = use_state(|| None::<String>);

    let notes_ref = use_node_ref();

    // Update form when existing_rating prop changes
    {
        let rating = rating.clone();
        let notes = notes.clone();
        let existing_rating = props.existing_rating.clone();
        use_effect_with(existing_rating, move |existing_rating| {
            if let Some(r) = existing_rating {
                rating.set(r.rating);
                notes.set(r.notes.clone().unwrap_or_default());
            }
            || ()
        });
    }

    let on_rate = {
        let rating = rating.clone();
        Callback::from(move |new_rating: i32| {
            rating.set(new_rating);
        })
    };

    let on_notes_change = {
        let notes = notes.clone();
        Callback::from(move |e: Event| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            notes.set(input.value());
        })
    };

    let on_submit = {
        let talk_id = props.talk_id.clone();
        let rating = rating.clone();
        let notes = notes.clone();
        let is_loading = is_loading.clone();
        let error = error.clone();
        let success_message = success_message.clone();
        let on_success = props.on_success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if *rating == 0 {
                error.set(Some("Please select a rating".to_string()));
                return;
            }

            let talk_id = talk_id.clone();
            let rating_value = *rating;
            let notes_value = if notes.trim().is_empty() {
                None
            } else {
                Some(notes.trim().to_string())
            };
            let is_loading = is_loading.clone();
            let error = error.clone();
            let success_message = success_message.clone();
            let on_success = on_success.clone();

            is_loading.set(true);
            error.set(None);
            success_message.set(None);

            spawn_local(async move {
                match RatingService::create_or_update_rating(&talk_id, rating_value, notes_value)
                    .await
                {
                    Ok(new_rating) => {
                        success_message.set(Some("Rating saved successfully!".to_string()));
                        if let Some(callback) = on_success {
                            callback.emit(new_rating);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to save rating: {}", e)));
                    }
                }
                is_loading.set(false);
            });
        })
    };

    let on_delete_click = {
        let talk_id = props.talk_id.clone();
        let is_loading = is_loading.clone();
        let error = error.clone();
        let success_message = success_message.clone();
        let rating = rating.clone();
        let notes = notes.clone();
        let on_delete = props.on_delete.clone();

        Callback::from(move |_: MouseEvent| {
            let talk_id = talk_id.clone();
            let is_loading = is_loading.clone();
            let error = error.clone();
            let success_message = success_message.clone();
            let rating = rating.clone();
            let notes = notes.clone();
            let on_delete = on_delete.clone();

            is_loading.set(true);
            error.set(None);
            success_message.set(None);

            spawn_local(async move {
                match RatingService::delete_rating(&talk_id).await {
                    Ok(()) => {
                        success_message.set(Some("Rating deleted successfully!".to_string()));
                        rating.set(0);
                        notes.set(String::new());
                        if let Some(callback) = on_delete {
                            callback.emit(());
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to delete rating: {}", e)));
                    }
                }
                is_loading.set(false);
            });
        })
    };

    let has_existing_rating = props.existing_rating.is_some();

    html! {
        <div class="rating-form">
            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label class="form-label">{"Your Rating"}</label>
                    <RatingStars
                        rating={*rating}
                        interactive={true}
                        on_rate={on_rate}
                        size="large"
                    />
                </div>

                <div class="form-group">
                    <label class="form-label" for="rating-notes">
                        {"Notes (optional)"}
                    </label>
                    <textarea
                        ref={notes_ref}
                        id="rating-notes"
                        class="notes-textarea"
                        value={(*notes).clone()}
                        onchange={on_notes_change}
                        placeholder="Add any comments about this talk..."
                        rows="3"
                    />
                </div>

                {
                    if let Some(err) = (*error).clone() {
                        html! { <div class="error-message">{ err }</div> }
                    } else {
                        html! {}
                    }
                }

                {
                    if let Some(msg) = (*success_message).clone() {
                        html! { <div class="success-message">{ msg }</div> }
                    } else {
                        html! {}
                    }
                }

                <div class="rating-form-actions">
                    <button
                        type="submit"
                        class="btn-primary"
                        disabled={*is_loading}
                    >
                        {
                            if *is_loading {
                                "Saving..."
                            } else if has_existing_rating {
                                "Update Rating"
                            } else {
                                "Submit Rating"
                            }
                        }
                    </button>

                    {
                        if has_existing_rating {
                            html! {
                                <button
                                    type="button"
                                    class="btn-danger"
                                    onclick={on_delete_click}
                                    disabled={*is_loading}
                                >
                                    { "Delete Rating" }
                                </button>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </form>
        </div>
    }
}
