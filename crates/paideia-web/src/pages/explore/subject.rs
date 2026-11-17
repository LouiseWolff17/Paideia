//! One subject: its details and the concepts inside it.

use leptos::prelude::*;
use leptos_router::{components::A, hooks::{use_navigate, use_params_map}};
use paideia_core::{Concept, ConceptInput, Subject, SubjectId, SubjectInput};

use super::components::{
    count, save, Breadcrumb, ConceptForm, DeleteButton, ErrorNote, LoadProblem, SubjectForm,
};
use crate::api;

pub fn concept_path(c: &Concept) -> String {
    format!("/explore/concepts/{}", c.id)
}

#[component]
pub fn SubjectPage() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.read().get("id").and_then(|s| s.parse::<SubjectId>().ok()));

    let subject = LocalResource::new(move || {
        let id = id.get();
        async move {
            match id {
                Some(id) => api::get_subject(id).await,
                None => Err("That subject doesn't exist.".into()),
            }
        }
    });

    view! {
        <section class="page">
            <Suspense fallback=|| view! { <p class="muted">"Loading…"</p> }>
                {move || {
                    subject
                        .get()
                        .map(|result| match result {
                            Err(e) => {
                                view! {
                                    <Breadcrumb trail=vec![] current=None />
                                    <LoadProblem message=e />
                                }
                                    .into_any()
                            }
                            Ok(s) => {
                                view! {
                                    <SubjectDetails
                                        subject=s
                                        on_changed=Callback::new(move |_| subject.refetch())
                                    />
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn SubjectDetails(subject: Subject, on_changed: Callback<()>) -> impl IntoView {
    let id = subject.id;
    let editing = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let navigate = use_navigate();

    let on_save = Callback::new(move |input: SubjectInput| {
        save(busy, error, api::update_subject(id, input), move |_| {
            editing.set(false);
            on_changed.run(());
        });
    });

    let warning = format!(
        "Delete \u{201c}{}\u{201d}? This also deletes {} and all of their material. This can't be undone.",
        subject.name,
        count(subject.concept_count, "concept", "concepts"),
    );
    let on_delete = Callback::new(move |_| {
        let navigate = navigate.clone();
        save(busy, error, api::delete_subject(id), move |_| navigate("/explore", Default::default()));
    });

    let initial = SubjectInput { name: subject.name.clone(), description: subject.description.clone() };

    view! {
        <Breadcrumb trail=vec![] current=Some(subject.name.clone()) />

        <Show
            when=move || editing.get()
            fallback={
                let subject = subject.clone();
                move || {
                    view! {
                        <header class="page-header">
                            <h1>{subject.name.clone()}</h1>
                            {(!subject.description.is_empty())
                                .then(|| view! { <p class="lead">{subject.description.clone()}</p> })}
                            <div class="actions">
                                <button
                                    type="button"
                                    class="btn secondary"
                                    on:click=move |_| {
                                        error.set(None);
                                        editing.set(true);
                                    }
                                >
                                    "Edit subject"
                                </button>
                            </div>
                        </header>
                    }
                }
            }
        >
            <h1>"Edit subject"</h1>
            <SubjectForm
                initial=initial.clone()
                submit_label="Save changes"
                on_save
                busy
                error
                on_cancel=Callback::new(move |_| {
                    error.set(None);
                    editing.set(false);
                })
            />
        </Show>

        <ConceptSection subject=id />

        <div class="danger-zone">
            <Show when=move || !editing.get()>
                <ErrorNote error />
            </Show>
            <DeleteButton label="Delete subject…" warning=warning.clone() on_confirm=on_delete busy />
        </div>
    }
}

#[component]
fn ConceptSection(subject: SubjectId) -> impl IntoView {
    let concepts = LocalResource::new(move || api::list_concepts(subject));
    let adding = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_save = Callback::new(move |input: ConceptInput| {
        save(busy, error, api::create_concept(subject, input), move |_| {
            adding.set(false);
            concepts.refetch();
        });
    });

    view! {
        <section class="section" aria-labelledby="concepts-heading">
            <h2 id="concepts-heading">"Concepts"</h2>
            <Suspense fallback=|| view! { <p class="muted">"Loading concepts…"</p> }>
                {move || {
                    concepts
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <LoadProblem message=e /> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <p class="muted">
                                        "No concepts yet. A concept is one idea or skill you want to understand, "
                                        "like \u{201c}eigenvalues\u{201d} or \u{201c}supply and demand\u{201d}."
                                    </p>
                                }
                                    .into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <ul class="item-list" aria-labelledby="concepts-heading">
                                        {list
                                            .into_iter()
                                            .map(|c| {
                                                view! {
                                                    <li>
                                                        <A href=concept_path(&c)>
                                                            <span class="item-title">{c.name.clone()}</span>
                                                            {(!c.summary.is_empty())
                                                                .then(|| {
                                                                    view! { <span class="item-text">{c.summary.clone()}</span> }
                                                                })}
                                                            <span class="item-meta">
                                                                {count(c.material_count, "piece of material", "pieces of material")}
                                                            </span>
                                                        </A>
                                                    </li>
                                                }
                                            })
                                            .collect_view()}
                                    </ul>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>

            <Show
                when=move || adding.get()
                fallback=move || {
                    view! {
                        <div class="actions">
                            <button type="button" class="btn primary" on:click=move |_| adding.set(true)>
                                "Add a concept"
                            </button>
                        </div>
                    }
                }
            >
                <ConceptForm
                    submit_label="Add concept"
                    on_save
                    busy
                    error
                    on_cancel=Callback::new(move |_| {
                        error.set(None);
                        adding.set(false);
                    })
                />
            </Show>
        </section>
    }
}
