use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;

#[component]
pub fn Home() -> impl IntoView {
    let subjects = LocalResource::new(api::list_subjects);

    view! {
        <section class="page">
            <h1>"Welcome to Paideia"</h1>
            <p class="lead">
                "A calm place to understand difficult ideas, remember them, "
                "and take one small step at a time."
            </p>

            // One clear suggested next action (design principle: clarity before density).
            <Suspense fallback=|| ()>
                {move || {
                    let has_subjects = subjects
                        .get()
                        .map(|r| r.map(|list| !list.is_empty()).unwrap_or(true))?;
                    Some(if has_subjects { view! { <TryLearn /> }.into_any() } else { view! { <AddFirstSubject /> }.into_any() })
                }}
            </Suspense>

            <p class="muted">
                "There are no streaks and nothing to keep up with. Come back whenever suits you."
            </p>
        </section>
    }
}

#[component]
fn AddFirstSubject() -> impl IntoView {
    view! {
        <div class="card next-step">
            <p class="eyebrow">"Suggested next step"</p>
            <h2>"Add your first subject"</h2>
            <p>
                "A subject is a course or anything you're studying. Inside it you can add the "
                "concepts you want to understand, and your own notes on each one."
            </p>
            <A href="/explore">"Open Explore"</A>
        </div>
    }
}

#[component]
fn TryLearn() -> impl IntoView {
    view! {
        <div class="card next-step">
            <p class="eyebrow">"Suggested next step"</p>
            <h2>"Learn with the tutor"</h2>
            <p>
                "Pick up a session where you left off, or open a concept and start one. "
                "You can go back, repeat or skip ahead whenever you like."
            </p>
            <A href="/learn">"Open Learn"</A>
        </div>
    }
}
