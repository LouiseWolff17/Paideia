use leptos::prelude::*;
use paideia_core::{Area, LearningStage, TeachingStrategy, TransitionCause};

/// The alternatives offered when something doesn't make sense yet (SDD §4.3).
const HELP_OPTIONS: [TeachingStrategy; 6] = [
    TeachingStrategy::ConcreteExample,
    TeachingStrategy::Simplification,
    TeachingStrategy::StepByStep,
    TeachingStrategy::Visualisation,
    TeachingStrategy::Analogy,
    TeachingStrategy::PrerequisiteReview,
];

#[component]
pub fn Learn() -> impl IntoView {
    let (stage, set_stage) = signal(LearningStage::first());
    let (help_open, set_help_open) = signal(false);
    let (chosen, set_chosen) = signal(None::<TeachingStrategy>);

    // Every move here is the learner's own choice, so the state machine allows
    // going back, repeating or skipping ahead.
    let go = move |target: LearningStage| {
        if let Ok(next) = stage
            .get_untracked()
            .transition_to(target, TransitionCause::LearnerChoice)
        {
            set_stage.set(next);
            set_help_open.set(false);
            set_chosen.set(None);
        }
    };

    view! {
        <section class="page">
            <h1>"Learn"</h1>
            <p class="lead">{Area::Learn.purpose()}</p>

            <div class="card soft">
                <p>
                    "This is a preview. Once you've added a subject and a concept, "
                    "each step will hold real explanations and exercises."
                </p>
            </div>

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

            <article class="card stage-card" aria-live="polite">
                <p class="eyebrow">
                    {move || format!("Step {} of {}", stage.get().index() + 1, LearningStage::ALL.len())}
                </p>
                <h2>{move || stage.get().label()}</h2>
                <p>{move || stage.get().description()}</p>
            </article>

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

            // Always available; no reason needed (SDD §4.3).
            <div class="help">
                <button
                    type="button"
                    class="btn gentle"
                    aria-expanded=move || help_open.get().to_string()
                    aria-controls="help-options"
                    on:click=move |_| set_help_open.update(|open| *open = !*open)
                >
                    "This doesn't make sense yet"
                </button>
                <Show when=move || help_open.get()>
                    <div id="help-options" class="card">
                        <p>"That's completely fine. No need to explain why — pick whatever sounds helpful:"</p>
                        <ul class="option-grid">
                            {HELP_OPTIONS
                                .into_iter()
                                .map(|t| {
                                    view! {
                                        <li>
                                            <button
                                                type="button"
                                                class="option"
                                                aria-pressed=move || (chosen.get() == Some(t)).to_string()
                                                on:click=move |_| set_chosen.set(Some(t))
                                            >
                                                <strong>{t.label()}</strong>
                                                <span>{t.description()}</span>
                                            </button>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                        <p class="muted" aria-live="polite">
                            {move || {
                                chosen
                                    .get()
                                    .map(|t| {
                                        format!(
                                            "In a real session, the tutor would now try: {}.",
                                            t.label().to_lowercase(),
                                        )
                                    })
                            }}
                        </p>
                    </div>
                </Show>
            </div>
        </section>
    }
}
