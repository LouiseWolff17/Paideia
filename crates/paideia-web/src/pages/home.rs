use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <section class="page">
            <h1>"Welcome to Paideia"</h1>
            <p class="lead">
                "A calm place to understand difficult ideas, remember them, "
                "and take one small step at a time."
            </p>

            // One clear suggested next action (design principle: clarity before density).
            <div class="card next-step">
                <p class="eyebrow">"Suggested next step"</p>
                <h2>"See how a learning session works"</h2>
                <p>
                    "Walk through the steps a session follows. You can go back, repeat "
                    "or skip ahead whenever you like."
                </p>
                <A href="/learn">"Open Learn"</A>
            </div>

            <p class="muted">
                "There are no streaks and nothing to keep up with. Come back whenever suits you."
            </p>
        </section>
    }
}
