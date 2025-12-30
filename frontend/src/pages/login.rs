use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;

use crate::{
    app::Route,
    services::auth::AuthService,
    types::LoginRequest,
};

#[function_component(Login)]
pub fn login() -> Html {
    let navigator = use_navigator().unwrap();
    let email = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    // Redirect to home if already authenticated
    {
        let navigator = navigator.clone();
        use_effect_with((), move |_| {
            if AuthService::is_authenticated() {
                navigator.push(&Route::Home);
            }
            || ()
        });
    }

    let email_clone = email.clone();
    let on_email_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        email_clone.set(input.value());
    });

    let password_clone = password.clone();
    let on_password_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        password_clone.set(input.value());
    });

    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let error = error.clone();
        let loading = loading.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let email = (*email).clone();
            let password = (*password).clone();
            let error = error.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();

            if email.is_empty() || password.is_empty() {
                error.set(Some("Please fill in all fields".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let request = LoginRequest { email, password };
                match AuthService::login(request).await {
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
                <h2>{ "Login" }</h2>
                <form onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="email">{ "Email" }</label>
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
                        <label for="password">{ "Password" }</label>
                        <input
                            type="password"
                            id="password"
                            value={(*password).clone()}
                            onchange={on_password_change}
                            disabled={*loading}
                            required=true
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
                        { if *loading { "Logging in..." } else { "Login" } }
                    </button>
                </form>

                <div class="oauth-divider">
                    <span>{ "or" }</span>
                </div>

                <a href="/api/auth/google" class="oauth-button google-button">
                    <span>{ "Continue with Google" }</span>
                </a>

                <a href="/api/auth/github" class="oauth-button github-button">
                    <span>{ "Continue with GitHub" }</span>
                </a>

                <a href="/api/auth/apple" class="oauth-button apple-button">
                    <span>{ "Continue with Apple" }</span>
                </a>

                <a href="/api/auth/facebook" class="oauth-button facebook-button">
                    <span>{ "Continue with Facebook" }</span>
                </a>

                <a href="/api/auth/linkedin" class="oauth-button linkedin-button">
                    <span>{ "Continue with LinkedIn" }</span>
                </a>

                <p class="auth-link">
                    { "Don't have an account? " }
                    <Link<Route> to={Route::Signup}>{ "Sign up" }</Link<Route>>
                </p>
            </div>
        </div>
    }
}
