use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};
use paideia_core::Area;

use crate::{
    api,
    pages::{
        ConceptPage, Explore, Home, Learn, NotFound, Placeholder, PreferencesPreview,
        ProgressPreview, SessionPage, SubjectPage,
    },
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <a class="skip-link" href="#main">"Skip to main content"</a>
            <div class="shell">
                <header class="sidebar">
                    <div class="brand">
                        <A href="/">
                            <span class="brand-mark" aria-hidden="true">"π"</span>
                            <span class="brand-name">"Paideia"</span>
                        </A>
                    </div>
                    <nav aria-label="Main">
                        <ul class="nav-list">
                            {Area::ALL
                                .into_iter()
                                .map(|area| view! { <li><A href=area.path()>{area.label()}</A></li> })
                                .collect_view()}
                        </ul>
                    </nav>
                    <ServerStatus />
                </header>

                <main id="main" tabindex="-1">
                    <Routes fallback=|| view! { <NotFound /> }>
                        <Route path=path!("/") view=Home />
                        <Route path=path!("/learn") view=Learn />
                        <Route path=path!("/learn/sessions/:id") view=SessionPage />
                        <Route
                            path=path!("/remember")
                            view=|| view! { <Placeholder area=Area::Remember when="milestone 5" /> }
                        />
                        <Route path=path!("/explore") view=Explore />
                        <Route path=path!("/explore/subjects/:id") view=SubjectPage />
                        <Route path=path!("/explore/concepts/:id") view=ConceptPage />
                        <Route
                            path=path!("/tasks")
                            view=|| view! { <Placeholder area=Area::MyTasks when="a later milestone" /> }
                        />
                        <Route path=path!("/progress") view=ProgressPreview />
                        <Route path=path!("/preferences") view=PreferencesPreview />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

/// A quiet line showing whether the local server (and AI) is available.
#[component]
fn ServerStatus() -> impl IntoView {
    let health = LocalResource::new(api::health);

    view! {
        <p class="server-status" role="status">
            <Suspense fallback=|| "Connecting to your local server…">
                {move || {
                    health
                        .get()
                        .map(|result| match result {
                            Ok(h) => {
                                let ai = if h.ai_configured {
                                    "AI tutor available"
                                } else {
                                    "AI not set up — everything else still works"
                                };
                                view! {
                                    <span class="dot ok" aria-hidden="true"></span>
                                    <span>"Running locally · v" {h.version} <br /> {ai}</span>
                                }
                                    .into_any()
                            }
                            Err(_) => {
                                view! {
                                    <span class="dot off" aria-hidden="true"></span>
                                    <span>"Can't reach the local server. Is it running?"</span>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
        </p>
    }
}
