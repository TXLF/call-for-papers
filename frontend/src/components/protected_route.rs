use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;
use crate::services::auth::AuthService;

#[derive(Properties, PartialEq)]
pub struct ProtectedRouteProps {
    pub children: Children,
}

#[function_component(ProtectedRoute)]
pub fn protected_route(props: &ProtectedRouteProps) -> Html {
    let navigator = use_navigator().unwrap();
    let is_authenticated = AuthService::is_authenticated();

    use_effect_with(is_authenticated, move |is_auth| {
        if !is_auth {
            navigator.push(&Route::Login);
        }
        || ()
    });

    if is_authenticated {
        html! {
            <>
                { for props.children.iter() }
            </>
        }
    } else {
        html! {
            <div class="loading">{ "Redirecting to login..." }</div>
        }
    }
}
