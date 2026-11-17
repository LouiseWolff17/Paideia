//! Learn: resume a session, or see how to start one.

mod activity;
mod consent;
mod session;
mod start;

use leptos::prelude::*;
use leptos_router::components::A;
use paideia_core::{catalog::Timestamp, Area, LearningStage, SessionSummary};

pub use consent::AiConsent;
pub use session::SessionPage;
pub use start::ConceptSessions;

use crate::{api, pages::explore::components::LoadProblem};

/// "today", "yesterday", "3 days ago". Calm and approximate on purpose.
pub fn when(t: Timestamp) -> String {
    let days = ((js_sys::Date::now() as i64 - t) / 86_400_000).max(0);
    match days {
        0 => "today".into(),
        1 => "yesterday".into(),
        n if n < 14 => format!("{n} days ago"),
        n if n < 60 => format!("{} weeks ago", n / 7),
        n => format!("{} months ago", n / 30),
    }
}

pub fn step_text(stage: LearningStage) -> String {
    format!("Step {} of {}: {}", stage.index() + 1, LearningStage::ALL.len(), stage.label())
}

/// A link to resume a session.
#[component]
pub fn SessionLink(summary: SessionSummary, #[prop(optional)] show_concept: bool) -> impl IntoView {
    let s = summary.session;
    let href = format!("/learn/sessions/{}", s.id);
    let title = if show_concept { summary.concept_name.clone() } else { format!("Session started {}", when(s.created_at)) };
    let mut meta = Vec::new();
    if show_concept {
        meta.push(summary.subject_name);
    }
    meta.push(step_text(s.stage));
    meta.push(format!("last active {}", when(s.updated_at)));

    view! {
        <a href=href>
            <span class="item-title">{title}</span>
            <span class="item-meta">{meta.join(" · ")}</span>
        </a>
    }
}

#[component]
pub fn Learn() -> impl IntoView {
    let recent = LocalResource::new(|| api::recent_sessions(5));
    let ai = LocalResource::new(api::ai_status);
    let ai_override = RwSignal::new(None);

    view! {
        <section class="page">
            <h1>"Learn"</h1>
            <p class="lead">{Area::Learn.purpose()}</p>

            <Suspense fallback=|| ()>
                {move || {
                    let status = ai_override.get().or_else(|| ai.get().and_then(Result::ok))?;
                    Some(view! { <AiConsent status on_change=Callback::new(move |s| ai_override.set(Some(s))) /> })
                }}
            </Suspense>

            <Suspense fallback=|| view! { <p class="muted">"Loading your sessions…"</p> }>
                {move || {
                    recent
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <LoadProblem message=e /> }.into_any(),
                            Ok(list) if list.is_empty() => view! { <StartHint /> }.into_any(),
                            Ok(list) => {
                                view! {
                                    <section class="section" aria-labelledby="recent-heading">
                                        <h2 id="recent-heading">"Pick up where you left off"</h2>
                                        <ul class="item-list">
                                            {list
                                                .into_iter()
                                                .map(|s| view! { <li><SessionLink summary=s show_concept=true /></li> })
                                                .collect_view()}
                                        </ul>
                                        <p class="muted">
                                            "To learn something new, open a concept in "
                                            <A href="/explore">"Explore"</A>
                                            " and start a session there."
                                        </p>
                                    </section>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>

            <details class="card how-it-works">
                <summary>"How a session works"</summary>
                <p>
                    "Each session follows one concept through these steps. You can go back, repeat a "
                    "step or skip ahead whenever you like, and ask for a different explanation at any point."
                </p>
                <ol class="plain-list">
                    {LearningStage::ALL
                        .into_iter()
                        .map(|s| view! { <li><strong>{s.label()}</strong>": "{s.description()}</li> })
                        .collect_view()}
                </ol>
            </details>
        </section>
    }
}

#[component]
fn StartHint() -> impl IntoView {
    view! {
        <div class="card next-step">
            <p class="eyebrow">"Suggested next step"</p>
            <h2>"Choose a concept to learn"</h2>
            <p>
                "Open a concept in Explore and choose \u{201c}Start a session\u{201d}. You decide which "
                "of your notes the tutor may use."
            </p>
            <A href="/explore">"Open Explore"</A>
        </div>
    }
}
