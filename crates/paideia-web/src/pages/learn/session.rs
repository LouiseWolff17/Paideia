//! One learning session: the steps, the tutor's replies, and the ways to ask
//! for something different.

use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
};
use paideia_core::{
    tutor::FOLLOW_UP_MAX_CHARS, ActivityRequest, AiStatus, LearningStage, SessionDetail, SessionId,
    TeachingStrategy,
};

use super::{activity::ActivityCard, consent::AiConsent, step_text};
use crate::{
    api,
    pages::explore::components::{count, save, DeleteButton, ErrorNote, LoadProblem},
};

/// The alternatives offered when something doesn't make sense yet (design doc §4).
const HELP_OPTIONS: [TeachingStrategy; 8] = [
    TeachingStrategy::ConcreteExample,
    TeachingStrategy::Simplification,
    TeachingStrategy::StepByStep,
    TeachingStrategy::Analogy,
    TeachingStrategy::Visualisation,
    TeachingStrategy::WorkedExample,
    TeachingStrategy::Counterexample,
    TeachingStrategy::PrerequisiteReview,
];

#[component]
pub fn SessionPage() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.read().get("id").and_then(|s| s.parse::<SessionId>().ok()));
    let detail = LocalResource::new(move || {
        let id = id.get();
        async move {
            match id {
                Some(id) => api::get_session(id).await,
                None => Err("That session doesn't exist.".into()),
            }
        }
    });

    view! {
        <section class="page">
            <Suspense fallback=|| view! { <p class="muted">"Loading your session…"</p> }>
                {move || {
                    detail
                        .get()
                        .map(|result| match result {
                            Err(e) => {
                                view! {
                                    <LoadProblem message=e />
                                    <p><A href="/learn">"Back to Learn"</A></p>
                                }
                                    .into_any()
                            }
                            Ok(d) => view! { <SessionView detail=d /> }.into_any(),
                        })
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn SessionView(detail: SessionDetail) -> impl IntoView {
    let summary = detail.summary;
    let id = summary.session.id;
    let stage = RwSignal::new(summary.session.stage);
    let activities = RwSignal::new(detail.activities);
    let thinking = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let help_open = RwSignal::new(false);
    let question = RwSignal::new(String::new());
    let deleting = RwSignal::new(false);

    let ai = RwSignal::new(None::<AiStatus>);
    spawn_local(async move {
        if let Ok(s) = api::ai_status().await {
            ai.try_set(Some(s));
        }
    });
    let ai_ready = move || ai.get().is_some_and(|s| s.configured && s.consented);
    let can_ask = move || ai_ready() && !thinking.get();

    let ask = move |request: ActivityRequest| {
        if thinking.get_untracked() {
            return;
        }
        save(thinking, error, api::ask_tutor(id, request), move |a| {
            activities.update(|v| v.push(a));
            help_open.set(false);
            question.set(String::new());
        });
    };

    // Every move here is the learner's own choice, so going back, repeating or
    // skipping ahead are all allowed. The server checks and saves it.
    let go = move |target: LearningStage| {
        let from = stage.get_untracked();
        if target == from {
            return;
        }
        stage.set(target);
        help_open.set(false);
        error.set(None);
        spawn_local(async move {
            if let Err(e) = api::set_stage(id, target).await {
                stage.try_set(from);
                error.try_set(Some(e));
            }
        });
    };

    let here = Memo::new(move |_| {
        let s = stage.get();
        activities.with(|v| v.iter().filter(|a| a.stage == s).cloned().collect::<Vec<_>>())
    });
    let elsewhere = Memo::new(move |_| {
        let s = stage.get();
        activities.with(|v| v.iter().filter(|a| a.stage != s).cloned().collect::<Vec<_>>())
    });

    let submit_question = move |ev: SubmitEvent| {
        ev.prevent_default();
        let q = question.get_untracked();
        if q.trim().is_empty() {
            error.set(Some("Type a question first.".into()));
            return;
        }
        ask(ActivityRequest { strategy: None, follow_up: Some(q) });
    };

    let navigate = use_navigate();
    let on_delete = Callback::new(move |_| {
        let navigate = navigate.clone();
        save(deleting, error, api::delete_session(id), move |_| navigate("/learn", Default::default()));
    });

    let shared = summary.session.material_ids.len() as u32;
    let subject_href = format!("/explore/subjects/{}", summary.subject_id);
    let concept_href = format!("/explore/concepts/{}", summary.session.concept_id);

    view! {
        <header class="page-header">
            <p class="eyebrow">"Learning session"</p>
            <h1>{summary.concept_name.clone()}</h1>
            <p class="item-meta">
                <A href=subject_href>{summary.subject_name.clone()}</A>
                " · "
                <A href=concept_href>"Concept and material"</A>
                " · "
                {if shared == 0 {
                    "no material shared with the tutor".to_owned()
                } else {
                    format!("{} shared with the tutor", count(shared, "piece of material", "pieces of material"))
                }}
            </p>
        </header>

        {move || {
            ai.get()
                .filter(|s| !(s.configured && s.consented))
                .map(|s| view! { <AiConsent status=s on_change=Callback::new(move |s| ai.set(Some(s))) /> })
        }}

        <nav aria-label="Session steps">
            <ol class="stepper">
                {LearningStage::ALL
                    .into_iter()
                    .map(|s| {
                        view! {
                            <li>
                                <button
                                    type="button"
                                    class="step"
                                    class:current=move || stage.get() == s
                                    class:done=move || s < stage.get()
                                    aria-current=move || (stage.get() == s).then_some("step")
                                    on:click=move |_| go(s)
                                >
                                    <span class="step-num" aria-hidden="true">{s.index() + 1}</span>
                                    <span>{s.label()}</span>
                                </button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ol>
        </nav>

        <article class="card stage-card">
            <p class="eyebrow">{move || step_text(stage.get())}</p>
            <h2>{move || stage.get().label()}</h2>
            <p>{move || stage.get().description()}</p>
            <Show when=move || stage.get() == LearningStage::IndependentPractice>
                <p class="muted">
                    "Try the problem on paper or in your head. Checking your answer here arrives "
                    "in a later update."
                </p>
            </Show>
        </article>

        <div class="activity-list" aria-live="polite">
            <For each=move || here.get() key=|a| a.id let:a>
                <ActivityCard activity=a />
            </For>
        </div>

        <Show when=move || thinking.get()>
            <p class="thinking" role="status">
                "The tutor is writing. This can take up to a minute."
            </p>
        </Show>
        <ErrorNote error />

        <Show
            when=move || !here.with(Vec::is_empty)
            fallback=move || {
                view! {
                    <div class="actions">
                        <button
                            type="button"
                            class="btn primary"
                            disabled=move || !can_ask()
                            on:click=move |_| ask(ActivityRequest::default())
                        >
                            "Begin this step with the tutor"
                        </button>
                    </div>
                }
            }
        >
            <form class="card form follow-up" on:submit=submit_question>
                <label>
                    <span>"Ask a question about this"</span>
                    <textarea
                        rows="2"
                        maxlength=FOLLOW_UP_MAX_CHARS
                        placeholder="e.g. Why does this only work for square matrices?"
                        bind:value=question
                    ></textarea>
                </label>
                <div class="actions">
                    <button type="submit" class="btn primary" disabled=move || !can_ask()>
                        "Ask"
                    </button>
                </div>
            </form>

            // Always available; no reason needed (design doc §4).
            <div class="help">
                <button
                    type="button"
                    class="btn gentle"
                    aria-expanded=move || help_open.get().to_string()
                    aria-controls="help-options"
                    on:click=move |_| help_open.update(|open| *open = !*open)
                >
                    "This doesn't make sense yet"
                </button>
                <Show when=move || help_open.get()>
                    <div id="help-options" class="card">
                        <p>"That's completely fine. No need to explain why. Pick whatever sounds helpful:"</p>
                        <ul class="option-grid">
                            {HELP_OPTIONS
                                .into_iter()
                                .map(|t| {
                                    view! {
                                        <li>
                                            <button
                                                type="button"
                                                class="option"
                                                disabled=move || !can_ask()
                                                on:click=move |_| {
                                                    ask(ActivityRequest { strategy: Some(t), follow_up: None })
                                                }
                                            >
                                                <strong>{t.label()}</strong>
                                                <span>{t.description()}</span>
                                            </button>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    </div>
                </Show>
            </div>
        </Show>

        <div class="actions">
            <button
                type="button"
                class="btn secondary"
                disabled=move || stage.get().previous().is_none()
                on:click=move |_| {
                    if let Some(p) = stage.get_untracked().previous() {
                        go(p)
                    }
                }
            >
                "Back"
            </button>
            <button
                type="button"
                class="btn primary"
                disabled=move || stage.get().next().is_none()
                on:click=move |_| {
                    if let Some(n) = stage.get_untracked().next() {
                        go(n)
                    }
                }
            >
                "Next step"
            </button>
        </div>

        <Show when=move || !elsewhere.with(Vec::is_empty)>
            <details class="earlier">
                <summary>
                    {move || format!("Other steps in this session ({})", elsewhere.with(Vec::len))}
                </summary>
                <div class="activity-list">
                    <For each=move || elsewhere.get() key=|a| a.id let:a>
                        <ActivityCard activity=a />
                    </For>
                </div>
            </details>
        </Show>

        <div class="danger-zone">
            <DeleteButton
                label="Delete this session…"
                warning="Delete this session and the tutor's replies in it? Your concept and material are kept. This can't be undone."
                    .to_string()
                on_confirm=on_delete
                busy=deleting
            />
        </div>
    }
}
