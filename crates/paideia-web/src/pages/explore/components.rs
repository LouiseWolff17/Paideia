//! Small building blocks shared by the Explore pages.

use std::future::Future;

use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local};
use leptos_router::components::A;
use paideia_core::{
    catalog::{
        DESCRIPTION_MAX_CHARS, MATERIAL_BODY_MAX_CHARS, MATERIAL_SOURCE_MAX_CHARS,
        MATERIAL_TITLE_MAX_CHARS, NAME_MAX_CHARS,
    },
    ConceptInput, MaterialInput, SubjectInput,
};

/// "1 concept", "3 concepts".
pub fn count(n: u32, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// Run a request, keeping `busy` and `error` up to date, then call `on_ok`.
/// Uses `try_set` because the page may have been left before the reply arrives.
pub fn save<T: 'static>(
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    request: impl Future<Output = Result<T, String>> + 'static,
    on_ok: impl FnOnce(T) + 'static,
) {
    busy.set(true);
    error.set(None);
    spawn_local(async move {
        match request.await {
            Ok(v) => on_ok(v),
            Err(e) => {
                error.try_set(Some(e));
            }
        }
        busy.try_set(false);
    });
}

/// A calm error line, announced to screen readers.
#[component]
pub fn ErrorNote(error: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || error.with(Option::is_some)>
            <p class="error-note" role="alert">{move || error.get()}</p>
        </Show>
    }
}

/// Shown while something loads, or when it can't be loaded.
#[component]
pub fn LoadProblem(message: String) -> impl IntoView {
    view! {
        <div class="card soft" role="alert">
            <p>{message}</p>
        </div>
    }
}

/// Explore › … › current page. `trail` holds (label, href) pairs after "Explore".
#[component]
pub fn Breadcrumb(trail: Vec<(String, String)>, current: Option<String>) -> impl IntoView {
    view! {
        <nav aria-label="Breadcrumb" class="breadcrumb">
            <ol>
                <li><A href="/explore" exact=true>"Explore"</A></li>
                {trail
                    .into_iter()
                    .map(|(label, href)| view! { <li><A href=href exact=true>{label}</A></li> })
                    .collect_view()}
                {current.map(|name| view! { <li aria-current="page">{name}</li> })}
            </ol>
        </nav>
    }
}

/// Shared buttons at the bottom of every form.
#[component]
fn FormActions(
    submit_label: &'static str,
    #[prop(into)] busy: Signal<bool>,
    on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="actions">
            <button type="submit" class="btn primary" disabled=move || busy.get()>
                {move || if busy.get() { "Saving…" } else { submit_label }}
            </button>
            {on_cancel
                .map(|cancel| {
                    view! {
                        <button type="button" class="btn secondary" on:click=move |_| cancel.run(())>
                            "Cancel"
                        </button>
                    }
                })}
        </div>
    }
}

#[component]
pub fn SubjectForm(
    #[prop(optional)] initial: SubjectInput,
    submit_label: &'static str,
    /// Called with trimmed, validated input.
    on_save: Callback<SubjectInput>,
    #[prop(into)] busy: Signal<bool>,
    error: RwSignal<Option<String>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let name = RwSignal::new(initial.name);
    let description = RwSignal::new(initial.description);

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let input = SubjectInput { name: name.get_untracked(), description: description.get_untracked() };
        match input.validated() {
            Ok(v) => on_save.run(v),
            Err(e) => error.set(Some(e.message)),
        }
    };

    view! {
        <form class="card form" on:submit=submit>
            <label>
                <span>"Name"</span>
                <input
                    type="text"
                    required
                    maxlength=NAME_MAX_CHARS
                    placeholder="e.g. Linear Algebra"
                    bind:value=name
                />
            </label>
            <label>
                <span>"Description " <small>"(optional)"</small></span>
                <textarea rows="3" maxlength=DESCRIPTION_MAX_CHARS bind:value=description></textarea>
            </label>
            <ErrorNote error />
            <FormActions submit_label busy on_cancel />
        </form>
    }
}

#[component]
pub fn ConceptForm(
    #[prop(optional)] initial: ConceptInput,
    submit_label: &'static str,
    on_save: Callback<ConceptInput>,
    #[prop(into)] busy: Signal<bool>,
    error: RwSignal<Option<String>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let name = RwSignal::new(initial.name);
    let summary = RwSignal::new(initial.summary);

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let input = ConceptInput { name: name.get_untracked(), summary: summary.get_untracked() };
        match input.validated() {
            Ok(v) => on_save.run(v),
            Err(e) => error.set(Some(e.message)),
        }
    };

    view! {
        <form class="card form" on:submit=submit>
            <label>
                <span>"Concept"</span>
                <input
                    type="text"
                    required
                    maxlength=NAME_MAX_CHARS
                    placeholder="e.g. Eigenvalues"
                    bind:value=name
                />
            </label>
            <label>
                <span>"Short summary " <small>"(optional)"</small></span>
                <textarea
                    rows="2"
                    maxlength=DESCRIPTION_MAX_CHARS
                    placeholder="A sentence on what this is about, in your own words."
                    bind:value=summary
                ></textarea>
            </label>
            <ErrorNote error />
            <FormActions submit_label busy on_cancel />
        </form>
    }
}

#[component]
pub fn MaterialForm(
    #[prop(optional)] initial: MaterialInput,
    submit_label: &'static str,
    on_save: Callback<MaterialInput>,
    #[prop(into)] busy: Signal<bool>,
    error: RwSignal<Option<String>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let title = RwSignal::new(initial.title);
    let source = RwSignal::new(initial.source);
    let body = RwSignal::new(initial.body);

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let input = MaterialInput {
            title: title.get_untracked(),
            source: source.get_untracked(),
            body: body.get_untracked(),
        };
        match input.validated() {
            Ok(v) => on_save.run(v),
            Err(e) => error.set(Some(e.message)),
        }
    };

    view! {
        <form class="card form" on:submit=submit>
            <label>
                <span>"Title"</span>
                <input
                    type="text"
                    required
                    maxlength=MATERIAL_TITLE_MAX_CHARS
                    placeholder="e.g. Lecture notes, week 3"
                    bind:value=title
                />
            </label>
            <label>
                <span>"Where it's from " <small>"(optional)"</small></span>
                <input
                    type="text"
                    maxlength=MATERIAL_SOURCE_MAX_CHARS
                    placeholder="e.g. Textbook chapter 4, or the lecture slides"
                    bind:value=source
                />
            </label>
            <label>
                <span>"Text"</span>
                <textarea
                    class="material-input"
                    rows="12"
                    required
                    maxlength=MATERIAL_BODY_MAX_CHARS
                    placeholder="Type or paste your notes, a definition, or a passage from your course."
                    bind:value=body
                ></textarea>
            </label>
            <p class="muted">"Saved only on this computer."</p>
            <ErrorNote error />
            <FormActions submit_label busy on_cancel />
        </form>
    }
}

/// A two-step delete: the first click explains what will be removed, and the
/// safe choice ("Keep it") comes first.
#[component]
pub fn DeleteButton(
    label: &'static str,
    /// What will be removed, shown before confirming.
    #[prop(into)]
    warning: Signal<String>,
    on_confirm: Callback<()>,
    #[prop(into)] busy: Signal<bool>,
) -> impl IntoView {
    let asking = RwSignal::new(false);

    view! {
        <Show
            when=move || asking.get()
            fallback=move || {
                view! {
                    <button type="button" class="btn gentle" on:click=move |_| asking.set(true)>
                        {label}
                    </button>
                }
            }
        >
            <div class="card confirm" role="group" aria-label="Confirm delete">
                <p>{move || warning.get()}</p>
                <div class="actions">
                    <button
                        type="button"
                        class="btn secondary"
                        autofocus
                        on:click=move |_| asking.set(false)
                    >
                        "Keep it"
                    </button>
                    <button
                        type="button"
                        class="btn danger"
                        disabled=move || busy.get()
                        on:click=move |_| on_confirm.run(())
                    >
                        {move || if busy.get() { "Deleting…" } else { "Yes, delete" }}
                    </button>
                </div>
            </div>
        </Show>
    }
}
