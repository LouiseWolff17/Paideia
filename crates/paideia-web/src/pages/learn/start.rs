//! On a concept's page: earlier sessions, and starting a new one with the
//! learner choosing exactly which material the tutor may see.

use std::collections::HashSet;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use paideia_core::{tutor::MATERIAL_PROMPT_BUDGET_CHARS, ConceptId, MaterialId};

use super::SessionLink;
use crate::{
    api,
    pages::explore::components::{save, ErrorNote, LoadProblem},
};

#[component]
pub fn ConceptSessions(concept: ConceptId) -> impl IntoView {
    let sessions = LocalResource::new(move || api::concept_sessions(concept));
    let choosing = RwSignal::new(false);

    view! {
        <section class="section" aria-labelledby="learn-heading">
            <h2 id="learn-heading">"Learn this concept"</h2>
            <Suspense fallback=|| ()>
                {move || {
                    sessions
                        .get()
                        .map(|r| match r {
                            Err(e) => view! { <LoadProblem message=e /> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <p class="muted">
                                        "Work through this concept step by step with the tutor, from a "
                                        "first example to practice."
                                    </p>
                                }
                                    .into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <ul class="item-list">
                                        {list
                                            .into_iter()
                                            .map(|s| view! { <li><SessionLink summary=s /></li> })
                                            .collect_view()}
                                    </ul>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
            <Show
                when=move || choosing.get()
                fallback=move || {
                    view! {
                        <div class="actions">
                            <button type="button" class="btn primary" on:click=move |_| choosing.set(true)>
                                "Start a session"
                            </button>
                        </div>
                    }
                }
            >
                <StartForm concept on_cancel=Callback::new(move |_| choosing.set(false)) />
            </Show>
        </section>
    }
}

#[component]
fn StartForm(concept: ConceptId, on_cancel: Callback<()>) -> impl IntoView {
    // Loaded when the form opens, so newly added material is always offered.
    let materials = LocalResource::new(move || api::list_materials(concept));
    let chosen = RwSignal::new(None::<HashSet<MaterialId>>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let navigate = use_navigate();

    // Everything is ticked to begin with; the learner can untick anything.
    Effect::new(move |_| {
        if let Some(Ok(list)) = materials.get() {
            if chosen.get_untracked().is_none() {
                chosen.set(Some(list.iter().map(|m| m.id).collect()));
            }
        }
    });

    let over_budget = move || {
        let picked = chosen.get().unwrap_or_default();
        let total: usize = materials
            .get()
            .and_then(Result::ok)
            .map(|list| list.iter().filter(|m| picked.contains(&m.id)).map(|m| m.body.chars().count()).sum())
            .unwrap_or(0);
        total > MATERIAL_PROMPT_BUDGET_CHARS
    };

    let start = move |_| {
        let mut ids: Vec<_> = chosen.get_untracked().unwrap_or_default().into_iter().collect();
        ids.sort();
        let navigate = navigate.clone();
        save(busy, error, api::start_session(concept, ids), move |s| {
            navigate(&format!("/learn/sessions/{}", s.session.id), Default::default())
        });
    };

    view! {
        <div class="card form">
            <h3>"Which material may the tutor use?"</h3>
            <p class="muted">
                "Ticked material is sent to Anthropic when you ask the tutor for help in this session. "
                "Unticked material stays on this computer."
            </p>
            <Suspense fallback=|| view! { <p class="muted">"Loading material…"</p> }>
                {move || {
                    materials
                        .get()
                        .map(|r| match r {
                            Err(e) => view! { <LoadProblem message=e /> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <p>
                                        "This concept has no material yet, so the tutor will explain from "
                                        "general knowledge. Adding your course notes first usually gives "
                                        "better, more relevant explanations."
                                    </p>
                                }
                                    .into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <fieldset class="checks">
                                        <legend class="visually-hidden">"Material to share"</legend>
                                        {list
                                            .into_iter()
                                            .map(|m| {
                                                let id = m.id;
                                                view! {
                                                    <label class="check">
                                                        <input
                                                            type="checkbox"
                                                            prop:checked=move || {
                                                                chosen.with(|c| c.as_ref().is_some_and(|c| c.contains(&id)))
                                                            }
                                                            on:change=move |ev| {
                                                                let on = event_target_checked(&ev);
                                                                chosen
                                                                    .update(|c| {
                                                                        let c = c.get_or_insert_with(HashSet::new);
                                                                        if on { c.insert(id); } else { c.remove(&id); }
                                                                    });
                                                            }
                                                        />
                                                        <span>
                                                            {m.title}
                                                            <small>
                                                                {format!(" · {} characters", m.body.chars().count())}
                                                            </small>
                                                        </span>
                                                    </label>
                                                }
                                            })
                                            .collect_view()}
                                    </fieldset>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
            <Show when=over_budget>
                <p class="error-note">
                    {format!(
                        "Together this is longer than about {} characters, so only the first part \
                         will be shared with each request. Untick some material to choose what matters most.",
                        MATERIAL_PROMPT_BUDGET_CHARS / 1000 * 1000,
                    )}
                </p>
            </Show>
            <ErrorNote error />
            <div class="actions">
                <button type="button" class="btn primary" disabled=move || busy.get() on:click=start>
                    {move || if busy.get() { "Starting…" } else { "Start session" }}
                </button>
                <button type="button" class="btn secondary" on:click=move |_| on_cancel.run(())>
                    "Cancel"
                </button>
            </div>
        </div>
    }
}
