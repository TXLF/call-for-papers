use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RatingStarsProps {
    /// Current rating value (1-5)
    pub rating: i32,
    /// Whether the stars are interactive (clickable)
    #[prop_or(false)]
    pub interactive: bool,
    /// Callback when a star is clicked (only fires if interactive=true)
    #[prop_or_default]
    pub on_rate: Option<Callback<i32>>,
    /// Size of the stars ("small", "medium", or "large")
    #[prop_or("medium".to_string())]
    pub size: String,
}

#[function_component(RatingStars)]
pub fn rating_stars(props: &RatingStarsProps) -> Html {
    let hover_rating = use_state(|| 0);

    let on_mouse_enter = {
        let hover_rating = hover_rating.clone();
        let interactive = props.interactive;
        Callback::from(move |star: i32| {
            if interactive {
                hover_rating.set(star);
            }
        })
    };

    let on_mouse_leave = {
        let hover_rating = hover_rating.clone();
        let interactive = props.interactive;
        Callback::from(move |_| {
            if interactive {
                hover_rating.set(0);
            }
        })
    };

    let on_click = {
        let on_rate = props.on_rate.clone();
        let interactive = props.interactive;
        Callback::from(move |star: i32| {
            if interactive {
                if let Some(callback) = &on_rate {
                    callback.emit(star);
                }
            }
        })
    };

    let size_class = match props.size.as_str() {
        "small" => "star-rating-small",
        "large" => "star-rating-large",
        _ => "star-rating-medium",
    };

    let display_rating = if *hover_rating > 0 {
        *hover_rating
    } else {
        props.rating
    };

    html! {
        <div
            class={classes!("star-rating", size_class, props.interactive.then_some("interactive"))}
            onmouseleave={on_mouse_leave}
        >
            {
                (1..=5).map(|star| {
                    let is_active = star <= display_rating;
                    let on_enter = on_mouse_enter.clone();
                    let on_click_inner = on_click.clone();

                    html! {
                        <span
                            key={star}
                            class={classes!("star", is_active.then_some("active"))}
                            onmouseenter={Callback::from(move |_| on_enter.emit(star))}
                            onclick={Callback::from(move |_| on_click_inner.emit(star))}
                            role={props.interactive.then_some("button")}
                            tabindex={props.interactive.then_some("0")}
                            aria-label={format!("Rate {} star{}", star, if star == 1 { "" } else { "s" })}
                        >
                            { "★" }
                        </span>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}
