use yew::prelude::*;
use gloo_net::http::Request;

#[function_component(Home)]
pub fn home() -> Html {
    let health_status = use_state(|| None::<String>);
    let db_health_status = use_state(|| None::<String>);

    let health_status_clone = health_status.clone();
    let check_health = Callback::from(move |_| {
        let health_status = health_status_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get("/api/health").send().await {
                Ok(resp) => {
                    if resp.ok() {
                        health_status.set(Some("OK".to_string()));
                    } else {
                        health_status.set(Some(format!("Error: {}", resp.status())));
                    }
                }
                Err(e) => health_status.set(Some(format!("Request failed: {}", e))),
            }
        });
    });

    let db_health_status_clone = db_health_status.clone();
    let check_db_health = Callback::from(move |_| {
        let db_health_status = db_health_status_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get("/api/health/db").send().await {
                Ok(resp) => {
                    if resp.ok() {
                        db_health_status.set(Some("OK".to_string()));
                    } else {
                        let text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        db_health_status.set(Some(format!("Error: {}", text)));
                    }
                }
                Err(e) => db_health_status.set(Some(format!("Request failed: {}", e))),
            }
        });
    });

    html! {
        <div class="card">
            <h2>{ "Welcome to Call for Papers" }</h2>
            <p>{ "An open source conference management system for speakers and organizers." }</p>

            <div style="margin-top: 2rem;">
                <h3>{ "System Status" }</h3>
                <div style="display: flex; gap: 1rem; margin-top: 1rem;">
                    <div>
                        <button onclick={check_health}>{ "Check API Health" }</button>
                        {
                            if let Some(status) = (*health_status).as_ref() {
                                html! { <p style="margin-top: 0.5rem;">{ format!("Status: {}", status) }</p> }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                    <div>
                        <button onclick={check_db_health}>{ "Check Database Health" }</button>
                        {
                            if let Some(status) = (*db_health_status).as_ref() {
                                html! { <p style="margin-top: 0.5rem;">{ format!("Status: {}", status) }</p> }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>
            </div>

            <div style="margin-top: 2rem;">
                <h3>{ "Features" }</h3>
                <ul style="margin-left: 1.5rem; margin-top: 0.5rem;">
                    <li>{ "Speaker talk submission" }</li>
                    <li>{ "Organizer review and rating system" }</li>
                    <li>{ "Conference scheduling" }</li>
                    <li>{ "Email communication" }</li>
                    <li>{ "AI-powered talk categorization" }</li>
                </ul>
            </div>
        </div>
    }
}
