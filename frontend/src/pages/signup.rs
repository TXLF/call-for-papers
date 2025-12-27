use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;

use crate::{
    app::Route,
    services::auth::AuthService,
    types::RegisterRequest,
};

#[function_component(Signup)]
pub fn signup() -> Html {
    let navigator = use_navigator().unwrap();
    let email = use_state(|| String::new());
    let username = use_state(|| String::new());
    let full_name = use_state(|| String::new());
    let password = use_state(|| String::new());
    let confirm_password = use_state(|| String::new());
    let bio = use_state(|| String::new());
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    let email_clone = email.clone();
    let on_email_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        email_clone.set(input.value());
    });

    let username_clone = username.clone();
    let on_username_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        username_clone.set(input.value());
    });

    let full_name_clone = full_name.clone();
    let on_full_name_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        full_name_clone.set(input.value());
    });

    let password_clone = password.clone();
    let on_password_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        password_clone.set(input.value());
    });

    let confirm_password_clone = confirm_password.clone();
    let on_confirm_password_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        confirm_password_clone.set(input.value());
    });

    let bio_clone = bio.clone();
    let on_bio_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        bio_clone.set(input.value());
    });

    let on_submit = {
        let email = email.clone();
        let username = username.clone();
        let full_name = full_name.clone();
        let password = password.clone();
        let confirm_password = confirm_password.clone();
        let bio = bio.clone();
        let error = error.clone();
        let loading = loading.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let email = (*email).clone();
            let username = (*username).clone();
            let full_name = (*full_name).clone();
            let password = (*password).clone();
            let confirm_password = (*confirm_password).clone();
            let bio = (*bio).clone();
            let error = error.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();

            // Validation
            if email.is_empty() || full_name.is_empty() || password.is_empty() {
                error.set(Some("Please fill in all required fields".to_string()));
                return;
            }

            if password != confirm_password {
                error.set(Some("Passwords do not match".to_string()));
                return;
            }

            if password.len() < 8 {
                error.set(Some("Password must be at least 8 characters".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let request = RegisterRequest {
                    email,
                    username: if username.is_empty() { None } else { Some(username) },
                    password,
                    full_name,
                    bio: if bio.is_empty() { None } else { Some(bio) },
                };

                match AuthService::register(request).await {
                    Ok(_) => {
                        navigator.push(&Route::Home);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="auth-container">
            <div class="auth-card">
                <h2>{ "Sign Up" }</h2>
                <form onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="email">{ "Email *" }</label>
                        <input
                            type="email"
                            id="email"
                            value={(*email).clone()}
                            onchange={on_email_change}
                            disabled={*loading}
                            required=true
                        />
                    </div>

                    <div class="form-group">
                        <label for="username">{ "Username (optional)" }</label>
                        <input
                            type="text"
                            id="username"
                            value={(*username).clone()}
                            onchange={on_username_change}
                            disabled={*loading}
                        />
                    </div>

                    <div class="form-group">
                        <label for="full_name">{ "Full Name *" }</label>
                        <input
                            type="text"
                            id="full_name"
                            value={(*full_name).clone()}
                            onchange={on_full_name_change}
                            disabled={*loading}
                            required=true
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">{ "Password *" }</label>
                        <input
                            type="password"
                            id="password"
                            value={(*password).clone()}
                            onchange={on_password_change}
                            disabled={*loading}
                            required=true
                        />
                        <small>{ "Minimum 8 characters" }</small>
                    </div>

                    <div class="form-group">
                        <label for="confirm_password">{ "Confirm Password *" }</label>
                        <input
                            type="password"
                            id="confirm_password"
                            value={(*confirm_password).clone()}
                            onchange={on_confirm_password_change}
                            disabled={*loading}
                            required=true
                        />
                    </div>

                    <div class="form-group">
                        <label for="bio">{ "Bio (optional)" }</label>
                        <textarea
                            id="bio"
                            value={(*bio).clone()}
                            onchange={on_bio_change}
                            disabled={*loading}
                            rows="3"
                        />
                    </div>

                    {
                        if let Some(err) = (*error).as_ref() {
                            html! {
                                <div class="error-message">
                                    { err }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }

                    <button type="submit" disabled={*loading}>
                        { if *loading { "Creating account..." } else { "Sign Up" } }
                    </button>
                </form>

                <p class="auth-link">
                    { "Already have an account? " }
                    <Link<Route> to={Route::Login}>{ "Login" }</Link<Route>>
                </p>
            </div>
        </div>
    }
}
