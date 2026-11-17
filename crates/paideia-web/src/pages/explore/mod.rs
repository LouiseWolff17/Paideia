//! Explore: browse and organise subjects, concepts and learning material.

mod components;
mod concept;
mod subject;

pub use concept::ConceptPage;
pub use subject::SubjectPage;

use leptos::prelude::*;
use leptos_router::components::A;
use paideia_core::{Area, Subject, SubjectInput};

use crate::api;
use components::{count, save, LoadProblem, SubjectForm};

pub fn subject_path(s: &Subject) -> String {
    format!("/explore/subjects/{}", s.id)
}

#[component]
pub fn Explore() -> impl IntoView {
    let subjects = LocalResource::new(api::list_subjects);

    view! {
        <section class="page">
            <h1>"Explore"</h1>
            <p class="lead">{Area::Explore.purpose()}</p>

            <Suspense fallback=|| view! { <p class="muted">"Loading your subjects…"</p> }>
                {move || {
                    subjects
                        .get()
                        .map(|result| match result {
                            Err(e) => view! { <LoadProblem message=e /> }.into_any(),
                            Ok(list) => {
                                view! {
                                    <SubjectList
                                        list
                                        on_added=Callback::new(move |_| subjects.refetch())
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
fn SubjectList(list: Vec<Subject>, on_added: Callback<()>) -> impl IntoView {
    let is_empty = list.is_empty();
    // With nothing here yet, show the form straight away: one clear next step.
    let adding = RwSignal::new(is_empty);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_save = Callback::new(move |input: SubjectInput| {
        save(busy, error, api::create_subject(input), move |_| {
            adding.set(false);
            on_added.run(());
        });
    });

    view! {
        {is_empty
            .then(|| {
                view! {
                    <div class="card soft">
                        <p>
                            "Nothing here yet. Start with a subject: a course you're taking, "
                            "or anything else you're studying."
                        </p>
                    </div>
                }
            })}

        {(!is_empty)
            .then(|| {
                view! {
                    <ul class="item-list" aria-label="Your subjects">
                        {list
                            .into_iter()
                            .map(|s| {
                                view! {
                                    <li>
                                        <A href=subject_path(&s)>
                                            <span class="item-title">{s.name.clone()}</span>
                                            {(!s.description.is_empty())
                                                .then(|| {
                                                    view! { <span class="item-text">{s.description.clone()}</span> }
                                                })}
                                            <span class="item-meta">
                                                {count(s.concept_count, "concept", "concepts")}
                                            </span>
                                        </A>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
            })}

        <Show
            when=move || adding.get()
            fallback=move || {
                view! {
                    <div class="actions">
                        <button type="button" class="btn primary" on:click=move |_| adding.set(true)>
                            "Add a subject"
                        </button>
                    </div>
                }
            }
        >
            <h2>"New subject"</h2>
            <SubjectForm
                submit_label="Add subject"
                on_save
                busy
                error
                on_cancel=Callback::new(move |_| {
                    error.set(None);
                    adding.set(false);
                })
            />
        </Show>
    }
}
