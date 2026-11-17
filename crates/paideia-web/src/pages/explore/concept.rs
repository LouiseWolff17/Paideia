//! One concept: its details and the learning material attached to it.

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use paideia_core::{Concept, ConceptId, ConceptInput, Material, MaterialInput};

use super::components::{
    count, save, Breadcrumb, ConceptForm, DeleteButton, ErrorNote, LoadProblem, MaterialForm,
};
use crate::api;

#[component]
pub fn ConceptPage() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.read().get("id").and_then(|s| s.parse::<ConceptId>().ok()));

    let concept = LocalResource::new(move || {
        let id = id.get();
        async move {
            match id {
                Some(id) => api::get_concept(id).await,
                None => Err("That concept doesn't exist.".into()),
            }
        }
    });

    view! {
        <section class="page">
            <Suspense fallback=|| view! { <p class="muted">"Loading…"</p> }>
                {move || {
                    concept
                        .get()
                        .map(|result| match result {
                            Err(e) => {
                                view! {
                                    <Breadcrumb trail=vec![] current=None />
                                    <LoadProblem message=e />
                                }
                                    .into_any()
                            }
                            Ok(c) => {
                                view! {
                                    <ConceptDetails
                                        concept=c
                                        on_changed=Callback::new(move |_| concept.refetch())
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

/// Breadcrumb with the parent subject's name, which needs its own request.
#[component]
fn ConceptBreadcrumb(concept: Concept) -> impl IntoView {
    let subject_id = concept.subject_id;
    let subject = LocalResource::new(move || api::get_subject(subject_id));
    let href = format!("/explore/subjects/{subject_id}");
    let name = concept.name;

    view! {
        <Suspense fallback={
            let name = name.clone();
            move || view! { <Breadcrumb trail=vec![] current=Some(name.clone()) /> }
        }>
            {
                let href = href.clone();
                let name = name.clone();
                move || {
                    let label = subject
                        .get()
                        .and_then(Result::ok)
                        .map(|s| s.name)
                        .unwrap_or_else(|| "Subject".into());
                    view! {
                        <Breadcrumb trail=vec![(label, href.clone())] current=Some(name.clone()) />
                    }
                }
            }
        </Suspense>
    }
}

#[component]
fn ConceptDetails(concept: Concept, on_changed: Callback<()>) -> impl IntoView {
    let id = concept.id;
    let subject_href = format!("/explore/subjects/{}", concept.subject_id);
    let editing = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let navigate = use_navigate();

    let on_save = Callback::new(move |input: ConceptInput| {
        save(busy, error, api::update_concept(id, input), move |_| {
            editing.set(false);
            on_changed.run(());
        });
    });

    let warning = format!(
        "Delete \u{201c}{}\u{201d}? This also deletes its {}. This can't be undone.",
        concept.name,
        count(concept.material_count, "piece of material", "pieces of material"),
    );
    let on_delete = Callback::new(move |_| {
        let navigate = navigate.clone();
        let to = subject_href.clone();
        save(busy, error, api::delete_concept(id), move |_| navigate(&to, Default::default()));
    });

    let initial = ConceptInput { name: concept.name.clone(), summary: concept.summary.clone() };

    view! {
        <ConceptBreadcrumb concept=concept.clone() />

        <Show
            when=move || editing.get()
            fallback={
                let concept = concept.clone();
                move || {
                    view! {
                        <header class="page-header">
                            <p class="eyebrow">"Concept"</p>
                            <h1>{concept.name.clone()}</h1>
                            {(!concept.summary.is_empty())
                                .then(|| view! { <p class="lead">{concept.summary.clone()}</p> })}
                            <div class="actions">
                                <button
                                    type="button"
                                    class="btn secondary"
                                    on:click=move |_| {
                                        error.set(None);
                                        editing.set(true);
                                    }
                                >
                                    "Edit concept"
                                </button>
                            </div>
                        </header>
                    }
                }
            }
        >
            <h1>"Edit concept"</h1>
            <ConceptForm
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

        <crate::pages::ConceptSessions concept=id />

        <MaterialSection concept=id />

        <div class="danger-zone">
            <Show when=move || !editing.get()>
                <ErrorNote error />
            </Show>
            <DeleteButton label="Delete concept…" warning=warning.clone() on_confirm=on_delete busy />
        </div>
    }
}

#[component]
fn MaterialSection(concept: ConceptId) -> impl IntoView {
    let materials = LocalResource::new(move || api::list_materials(concept));
    let adding = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let refresh = Callback::new(move |_| materials.refetch());

    let on_save = Callback::new(move |input: MaterialInput| {
        save(busy, error, api::create_material(concept, input), move |_| {
            adding.set(false);
            materials.refetch();
        });
    });

    view! {
        <section class="section" aria-labelledby="material-heading">
            <h2 id="material-heading">"Learning material"</h2>
            <Suspense fallback=|| view! { <p class="muted">"Loading material…"</p> }>
                {move || {
                    materials
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <LoadProblem message=e /> }.into_any(),
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <p class="muted">
                                        "No material yet. Paste in your notes, a definition or a passage "
                                        "from your course. The tutor will use it to ground its explanations."
                                    </p>
                                }
                                    .into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <ul class="material-list">
                                        {list
                                            .into_iter()
                                            .map(|m| view! { <li><MaterialCard material=m on_changed=refresh /></li> })
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
                                "Add text"
                            </button>
                        </div>
                    }
                }
            >
                <MaterialForm
                    submit_label="Save text"
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

#[component]
fn MaterialCard(material: Material, on_changed: Callback<()>) -> impl IntoView {
    let id = material.id;
    let editing = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_save = Callback::new(move |input: MaterialInput| {
        save(busy, error, api::update_material(id, input), move |_| {
            editing.set(false);
            on_changed.run(());
        });
    });
    let on_delete = Callback::new(move |_| {
        save(busy, error, api::delete_material(id), move |_| on_changed.run(()));
    });

    let warning = format!("Delete \u{201c}{}\u{201d}? This can't be undone.", material.title);
    let chars = material.body.chars().count();
    let initial = MaterialInput {
        title: material.title.clone(),
        source: material.source.clone(),
        body: material.body.clone(),
    };

    view! {
        <Show
            when=move || editing.get()
            fallback={
                let m = material.clone();
                let warning = warning.clone();
                move || {
                    view! {
                        <article class="card material">
                            <h3>{m.title.clone()}</h3>
                            <p class="item-meta">
                                {m.kind.label()}
                                {(!m.source.is_empty()).then(|| format!(" · from {}", m.source))}
                            </p>
                            <details>
                                <summary>{format!("Show text ({chars} characters)")}</summary>
                                <div class="material-body">{m.body.clone()}</div>
                            </details>
                            <ErrorNote error />
                            <div class="actions">
                                <button
                                    type="button"
                                    class="btn secondary"
                                    on:click=move |_| {
                                        error.set(None);
                                        editing.set(true);
                                    }
                                >
                                    "Edit"
                                </button>
                                <DeleteButton label="Delete…" warning=warning.clone() on_confirm=on_delete busy />
                            </div>
                        </article>
                    }
                }
            }
        >
            <MaterialForm
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
    }
}
