use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{
    home::Home, login::Login, my_talks::MyTalks, not_found::NotFound, signup::Signup,
    submit_talk::SubmitTalk,
};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/signup")]
    Signup,
    #[at("/talks/submit")]
    SubmitTalk,
    #[at("/talks/mine")]
    MyTalks,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Login => html! { <Login /> },
        Route::Signup => html! { <Signup /> },
        Route::SubmitTalk => html! { <SubmitTalk /> },
        Route::MyTalks => html! { <MyTalks /> },
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
                        <Link<Route> to={Route::MyTalks}>{ "My Talks" }</Link<Route>>
                        <Link<Route> to={Route::SubmitTalk}>{ "Submit Talk" }</Link<Route>>
                        <Link<Route> to={Route::Login}>{ "Login" }</Link<Route>>
                        <Link<Route> to={Route::Signup}>{ "Sign Up" }</Link<Route>>
                    </nav>
                </header>
                <main>
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}
