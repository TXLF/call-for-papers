use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{home::Home, not_found::NotFound};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <div id="app">
                <header>
                    <h1>{ "Call for Papers" }</h1>
                    <nav>
                        <Link<Route> to={Route::Home}>{ "Home" }</Link<Route>>
                    </nav>
                </header>
                <main>
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}
