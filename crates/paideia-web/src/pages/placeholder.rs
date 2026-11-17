use leptos::prelude::*;
use leptos_router::components::A;
use paideia_core::{Area, EvidenceDimension};

use crate::{api, pages::AiConsent};

/// Allow or stop AI requests.
#[component]
fn AiSettings() -> impl IntoView {
    let status = LocalResource::new(api::ai_status);
    let latest = RwSignal::new(None);
    view! {
        <Suspense fallback=|| ()>
            {move || {
                let s = latest.get().or_else(|| status.get().and_then(Result::ok))?;
                Some(view! { <AiConsent status=s always=true on_change=Callback::new(move |s| latest.set(Some(s))) /> })
            }}
        </Suspense>
    }
}

/// A calm stand-in for an area that is not built yet.
#[component]
pub fn Placeholder(
    area: Area,
    when: &'static str,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <section class="page">
            <h1>{area.label()}</h1>
            <p class="lead">{area.purpose()}</p>
            {children.map(|c| c())}
            <div class="card soft">
                <p>"This area arrives in " {when} "."</p>
            </div>
        </section>
    }
}

#[component]
pub fn ProgressPreview() -> impl IntoView {
    view! {
        <Placeholder area=Area::Progress when="milestone 5">
            <div class="card">
                <h2>"How progress will be shown"</h2>
                <p>
                    "Not as one score. Paideia keeps separate evidence for each of these, "
                    "and \u{201c}not enough evidence yet\u{201d} is never treated as \u{201c}doesn't understand\u{201d}."
                </p>
                <ul class="plain-list">
                    {EvidenceDimension::ALL
                        .into_iter()
                        .map(|d| view! { <li>{d.question()}</li> })
                        .collect_view()}
                </ul>
            </div>
        </Placeholder>
    }
}

#[component]
pub fn PreferencesPreview() -> impl IntoView {
    view! {
        <Placeholder area=Area::Preferences when="a later milestone">
            <div class="card">
                <h2>"Your data and the AI tutor"</h2>
                <p>
                    "Your subjects, notes and learning history are stored on this computer. "
                    "Every feature apart from the tutor's explanations works without AI."
                </p>
            </div>
            <AiSettings />
        </Placeholder>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section class="page">
            <h1>"This page doesn't exist"</h1>
            <p>
                "Nothing is lost. " <A href="/">"Go back to the start"</A>
            </p>
        </section>
    }
}
