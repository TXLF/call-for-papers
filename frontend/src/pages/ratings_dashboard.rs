use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    components::RatingStars,
    services::ratings::RatingService,
    types::RatingsStatisticsResponse,
};

#[function_component(RatingsDashboard)]
pub fn ratings_dashboard() -> Html {
    let statistics = use_state(|| None::<RatingsStatisticsResponse>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Fetch statistics on mount
    {
        let statistics = statistics.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                loading.set(true);
                error.set(None);

                match RatingService::get_statistics().await {
                    Ok(stats) => {
                        statistics.set(Some(stats));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="ratings-dashboard">
            <h2>{ "Ratings Dashboard" }</h2>

            {
                if *loading {
                    html! {
                        <div class="loading">{ "Loading statistics..." }</div>
                    }
                } else if let Some(err) = (*error).as_ref() {
                    html! {
                        <div class="error-message">
                            { err }
                        </div>
                    }
                } else if let Some(stats) = (*statistics).as_ref() {
                    html! {
                        <>
                            <div class="stats-overview">
                                <div class="stat-card">
                                    <div class="stat-value">{ stats.total_talks }</div>
                                    <div class="stat-label">{ "Total Talks" }</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{ stats.total_ratings }</div>
                                    <div class="stat-label">{ "Total Ratings" }</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{ stats.talks_with_ratings }</div>
                                    <div class="stat-label">{ "Talks Rated" }</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{ stats.talks_without_ratings }</div>
                                    <div class="stat-label">{ "Unrated Talks" }</div>
                                </div>
                                {
                                    if let Some(avg) = stats.overall_average_rating {
                                        html! {
                                            <div class="stat-card highlight">
                                                <div class="stat-value">{ format!("{:.1}", avg) }</div>
                                                <div class="stat-label">{ "Average Rating" }</div>
                                                <RatingStars rating={avg.round() as i32} interactive={false} size="small" />
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>

                            <div class="rating-distribution-section">
                                <h3>{ "Rating Distribution" }</h3>
                                <div class="distribution-chart">
                                    {
                                        for [5, 4, 3, 2, 1].iter().map(|star| {
                                            let count = match star {
                                                5 => stats.rating_distribution.five_star,
                                                4 => stats.rating_distribution.four_star,
                                                3 => stats.rating_distribution.three_star,
                                                2 => stats.rating_distribution.two_star,
                                                1 => stats.rating_distribution.one_star,
                                                _ => 0,
                                            };
                                            let percentage = if stats.total_ratings > 0 {
                                                (count as f64 / stats.total_ratings as f64) * 100.0
                                            } else {
                                                0.0
                                            };

                                            html! {
                                                <div class="distribution-row" key={*star}>
                                                    <div class="distribution-label">
                                                        <RatingStars rating={*star} interactive={false} size="small" />
                                                    </div>
                                                    <div class="distribution-bar-container">
                                                        <div
                                                            class="distribution-bar"
                                                            style={format!("width: {}%", percentage)}
                                                        />
                                                    </div>
                                                    <div class="distribution-count">
                                                        { format!("{} ({:.0}%)", count, percentage) }
                                                    </div>
                                                </div>
                                            }
                                        })
                                    }
                                </div>
                            </div>

                            <div class="talk-stats-section">
                                <h3>{ "Talk Ratings" }</h3>
                                <div class="talk-stats-table">
                                    {
                                        for stats.talk_stats.iter().map(|talk_stat| {
                                            let state_class = format!("state-{}", talk_stat.state.to_lowercase());

                                            html! {
                                                <div class="talk-stat-row" key={talk_stat.talk_id.clone()}>
                                                    <div class="talk-stat-title">
                                                        <strong>{ &talk_stat.talk_title }</strong>
                                                        <div class="talk-stat-speaker">
                                                            { "by " }{ &talk_stat.speaker_name }
                                                        </div>
                                                        <span class={classes!("talk-status", state_class)}>
                                                            { &talk_stat.state }
                                                        </span>
                                                    </div>
                                                    <div class="talk-stat-rating">
                                                        {
                                                            if let Some(avg) = talk_stat.average_rating {
                                                                html! {
                                                                    <>
                                                                        <div class="avg-rating-display">
                                                                            <span class="avg-rating-number">
                                                                                { format!("{:.1}", avg) }
                                                                            </span>
                                                                            <RatingStars rating={avg.round() as i32} interactive={false} size="small" />
                                                                        </div>
                                                                        <div class="rating-count-display">
                                                                            { format!("({} rating{})", talk_stat.rating_count, if talk_stat.rating_count == 1 { "" } else { "s" }) }
                                                                        </div>
                                                                    </>
                                                                }
                                                            } else {
                                                                html! {
                                                                    <div class="no-ratings">{ "No ratings yet" }</div>
                                                                }
                                                            }
                                                        }
                                                    </div>
                                                </div>
                                            }
                                        })
                                    }
                                </div>
                            </div>
                        </>
                    }
                } else {
                    html! {
                        <div class="empty-state">
                            <p>{ "No data available" }</p>
                        </div>
                    }
                }
            }
        </div>
    }
}
