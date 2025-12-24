use yew::prelude::*;
use yew_router::prelude::*;
use crate::app::Route;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <div class="card">
            <h2>{ "404 - Page Not Found" }</h2>
            <p>{ "The page you're looking for doesn't exist." }</p>
            <Link<Route> to={Route::Home}>
                <button style="margin-top: 1rem;">{ "Go Home" }</button>
            </Link<Route>>
        </div>
    }
}
