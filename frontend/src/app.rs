use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{protected_route::ProtectedRoute, organizer_route::OrganizerRoute};
use crate::pages::{
    home::Home, login::Login, my_talks::MyTalks, not_found::NotFound, signup::Signup,
    submit_talk::SubmitTalk, speaker_dashboard::SpeakerDashboard, organizer_talks::OrganizerTalks,
    organizer_dashboard::OrganizerDashboard, organizer_labels::OrganizerLabels,
    ratings_dashboard::RatingsDashboard, manage_tracks::ManageTracks,
    manage_schedule_slots::ManageScheduleSlots, assign_talks::AssignTalks,
};
use crate::services::auth::AuthService;

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
    #[at("/speaker/dashboard")]
    SpeakerDashboard,
    #[at("/organizer/dashboard")]
    OrganizerDashboard,
    #[at("/organizer/talks")]
    OrganizerTalks,
    #[at("/organizer/labels")]
    OrganizerLabels,
    #[at("/organizer/ratings")]
    RatingsDashboard,
    #[at("/organizer/tracks")]
    ManageTracks,
    #[at("/organizer/schedule-slots")]
    ManageScheduleSlots,
    #[at("/organizer/assign-talks")]
    AssignTalks,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Login => html! { <Login /> },
        Route::Signup => html! { <Signup /> },
        Route::SubmitTalk => html! {
            <ProtectedRoute>
                <SubmitTalk />
            </ProtectedRoute>
        },
        Route::MyTalks => html! {
            <ProtectedRoute>
                <MyTalks />
            </ProtectedRoute>
        },
        Route::SpeakerDashboard => html! {
            <ProtectedRoute>
                <SpeakerDashboard />
            </ProtectedRoute>
        },
        Route::OrganizerDashboard => html! {
            <OrganizerRoute>
                <OrganizerDashboard />
            </OrganizerRoute>
        },
        Route::OrganizerTalks => html! {
            <OrganizerRoute>
                <OrganizerTalks />
            </OrganizerRoute>
        },
        Route::OrganizerLabels => html! {
            <OrganizerRoute>
                <OrganizerLabels />
            </OrganizerRoute>
        },
        Route::RatingsDashboard => html! {
            <OrganizerRoute>
                <RatingsDashboard />
            </OrganizerRoute>
        },
        Route::ManageTracks => html! {
            <OrganizerRoute>
                <ManageTracks />
            </OrganizerRoute>
        },
        Route::ManageScheduleSlots => html! {
            <OrganizerRoute>
                <ManageScheduleSlots />
            </OrganizerRoute>
        },
        Route::AssignTalks => html! {
            <OrganizerRoute>
                <AssignTalks />
            </OrganizerRoute>
        },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(AppContent)]
fn app_content() -> Html {
    let is_authenticated = use_state(|| AuthService::is_authenticated());
    let is_organizer = use_state(|| AuthService::is_organizer());
    let navigator = use_navigator().unwrap();

    // Check authentication status on mount and when it might change
    {
        let is_authenticated = is_authenticated.clone();
        let is_organizer = is_organizer.clone();
        use_effect_with((), move |_| {
            is_authenticated.set(AuthService::is_authenticated());
            is_organizer.set(AuthService::is_organizer());
            || ()
        });
    }

    let on_logout = {
        let is_authenticated = is_authenticated.clone();
        let is_organizer = is_organizer.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            AuthService::logout();
            is_authenticated.set(false);
            is_organizer.set(false);
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div id="app">
            <header>
                <h1>{ "Call for Papers" }</h1>
                <nav>
                    <Link<Route> to={Route::Home}>{ "Home" }</Link<Route>>
                    if *is_authenticated {
                        <>
                            <Link<Route> to={Route::SpeakerDashboard}>{ "Dashboard" }</Link<Route>>
                            <Link<Route> to={Route::MyTalks}>{ "My Talks" }</Link<Route>>
                            <Link<Route> to={Route::SubmitTalk}>{ "Submit Talk" }</Link<Route>>
                            if *is_organizer {
                                <Link<Route> to={Route::OrganizerDashboard}>{ "Dashboard" }</Link<Route>>
                                <Link<Route> to={Route::OrganizerTalks}>{ "Review Talks" }</Link<Route>>
                                <Link<Route> to={Route::OrganizerLabels}>{ "Manage Labels" }</Link<Route>>
                                <Link<Route> to={Route::ManageTracks}>{ "Manage Tracks" }</Link<Route>>
                                <Link<Route> to={Route::ManageScheduleSlots}>{ "Manage Time Slots" }</Link<Route>>
                                <Link<Route> to={Route::AssignTalks}>{ "Assign Talks" }</Link<Route>>
                                <Link<Route> to={Route::RatingsDashboard}>{ "Ratings Dashboard" }</Link<Route>>
                            }
                            <button onclick={on_logout} class="logout-button">{ "Logout" }</button>
                        </>
                    } else {
                        <>
                            <Link<Route> to={Route::Login}>{ "Login" }</Link<Route>>
                            <Link<Route> to={Route::Signup}>{ "Sign Up" }</Link<Route>>
                        </>
                    }
                </nav>
            </header>
            <main>
                <Switch<Route> render={switch} />
            </main>
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <AppContent />
        </BrowserRouter>
    }
}
