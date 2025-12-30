use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::window;

use crate::{
    app::Route,
    services::auth::AuthService,
};

#[function_component(AuthCallback)]
pub fn auth_callback() -> Html {
    let navigator = use_navigator().unwrap();
    let error = use_state(|| None::<String>);

    {
        let navigator = navigator.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            // Get the token from the URL query parameters
            if let Some(window) = window() {
                if let Ok(location) = window.location().search() {
                    // Parse query parameters
                    let params = web_sys::UrlSearchParams::new_with_str(&location).ok();

                    if let Some(params) = params {
                        if let Some(token) = params.get("token") {
                            // Store the token
                            AuthService::set_token(&token);

                            // Redirect to home
                            navigator.push(&Route::Home);
                        } else {
                            error.set(Some("No authentication token received".to_string()));
                        }
                    } else {
                        error.set(Some("Failed to parse callback parameters".to_string()));
                    }
                } else {
                    error.set(Some("Failed to read callback URL".to_string()));
                }
            }

            || ()
        });
    }

    html! {
        <div class="auth-container">
            <div class="auth-card">
                {
                    if let Some(err) = (*error).as_ref() {
                        html! {
                            <>
                                <h2>{ "Authentication Error" }</h2>
                                <div class="error-message">
                                    { err }
                                </div>
                                <p class="auth-link">
                                    <Link<Route> to={Route::Login}>{ "Return to Login" }</Link<Route>>
                                </p>
                            </>
                        }
                    } else {
                        html! {
                            <>
                                <h2>{ "Authenticating..." }</h2>
                                <p>{ "Please wait while we complete your authentication." }</p>
                            </>
                        }
                    }
                }
            </div>
        </div>
    }
}
