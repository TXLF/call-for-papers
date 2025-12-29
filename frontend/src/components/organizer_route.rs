use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;
use crate::services::auth::AuthService;

#[derive(Properties, PartialEq)]
pub struct OrganizerRouteProps {
    pub children: Children,
}

#[function_component(OrganizerRoute)]
pub fn organizer_route(props: &OrganizerRouteProps) -> Html {
    let navigator = use_navigator().unwrap();
    let is_authenticated = AuthService::is_authenticated();
    let is_organizer = AuthService::is_organizer();

    use_effect_with((is_authenticated, is_organizer), move |(is_auth, is_org)| {
        if !is_auth {
            navigator.push(&Route::Login);
        } else if !is_org {
            navigator.push(&Route::Home);
        }
        || ()
    });

    if is_authenticated && is_organizer {
        html! {
            <>
                { for props.children.iter() }
            </>
        }
    } else if !is_authenticated {
        html! {
            <div class="loading">{ "Redirecting to login..." }</div>
        }
    } else {
        html! {
            <div class="error-message">
                <p>{ "Access Denied: Organizer privileges required" }</p>
                <p>{ "Redirecting to home..." }</p>
            </div>
        }
    }
}
