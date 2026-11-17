//! Telling the learner, before the first AI request, what leaves the device
//! (design doc §8, "Local-first privacy").

use leptos::prelude::*;
use paideia_core::AiStatus;

use crate::{
    api,
    pages::explore::components::{save, ErrorNote},
};

/// Shows whether the AI tutor can be used, and lets the learner allow or stop
/// AI requests. When `always` is false, nothing is shown once AI is allowed.
#[component]
pub fn AiConsent(
    status: AiStatus,
    on_change: Callback<AiStatus>,
    #[prop(optional)] always: bool,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let set = move |consented: bool| {
        save(busy, error, api::set_ai_consent(consented), move |s| on_change.run(s));
    };

    if !status.configured {
        return view! {
            <div class="card soft">
                <h2>"The AI tutor isn't set up"</h2>
                <p>
                    "To use it, add an Anthropic API key to the "<code>".env"</code>" file and restart "
                    "the local server. You can still move through every step, and everything else in "
                    "Paideia works without it."
                </p>
            </div>
        }
        .into_any();
    }

    if status.consented {
        return always
            .then(|| {
                view! {
                    <div class="card">
                        <h2>"AI requests are allowed"</h2>
                        <WhatIsSent />
                        <ErrorNote error />
                        <div class="actions">
                            <button
                                type="button"
                                class="btn secondary"
                                disabled=move || busy.get()
                                on:click=move |_| set(false)
                            >
                                "Stop allowing AI requests"
                            </button>
                        </div>
                    </div>
                }
            })
            .into_any();
    }

    view! {
        <div class="card next-step consent">
            <p class="eyebrow">"Before the tutor helps"</p>
            <h2>"How AI requests work"</h2>
            <WhatIsSent />
            <ErrorNote error />
            <div class="actions">
                <button
                    type="button"
                    class="btn primary"
                    disabled=move || busy.get()
                    on:click=move |_| set(true)
                >
                    "I understand, allow AI requests"
                </button>
            </div>
            <p class="muted">"You can change this at any time in Preferences."</p>
        </div>
    }
    .into_any()
}

#[component]
fn WhatIsSent() -> impl IntoView {
    view! {
        <p>
            "Your subjects, notes and learning history are stored on this computer. When you ask the "
            "tutor for help, the text needed for that one request is sent to Anthropic, an external "
            "AI provider, to be processed. Nothing is sent until you press a button that asks the tutor."
        </p>
        <div class="two-lists">
            <div>
                <h3>"Sent with a request"</h3>
                <ul class="plain-list">
                    <li>"The subject and concept names, and your summary"</li>
                    <li>"The material you chose to share for that session"</li>
                    <li>"Your question, if you typed one"</li>
                    <li>"The titles of earlier replies in the same session"</li>
                </ul>
            </div>
            <div>
                <h3>"Never sent"</h3>
                <ul class="plain-list">
                    <li>"Material you didn't choose to share"</li>
                    <li>"Other concepts, subjects and sessions"</li>
                    <li>"Your API key (it stays on the local server)"</li>
                </ul>
            </div>
        </div>
    }
}
