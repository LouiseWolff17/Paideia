//! One tutor reply, shown with what it was asked to do and why.

use leptos::prelude::*;
use paideia_core::Activity;

#[component]
pub fn ActivityCard(activity: Activity) -> impl IntoView {
    let c = activity.content;
    let label = activity.strategy.map(|s| s.label()).unwrap_or_else(|| activity.stage.label());

    view! {
        <article class="card activity">
            {activity
                .follow_up
                .map(|q| view! { <p class="you-asked"><span>"You asked: "</span>{q}</p> })}
            <p class="eyebrow">{label}</p>
            <h3>{c.title}</h3>
            {c.paragraphs.into_iter().map(|p| view! { <p>{p}</p> }).collect_view()}
            {(!c.steps.is_empty())
                .then(|| {
                    view! {
                        <ol class="steps">
                            {c.steps.into_iter().map(|s| view! { <li>{s}</li> }).collect_view()}
                        </ol>
                    }
                })}
            {(!c.key_points.is_empty())
                .then(|| {
                    view! {
                        <div class="key-points">
                            <h4>"Key points"</h4>
                            <ul>{c.key_points.into_iter().map(|k| view! { <li>{k}</li> }).collect_view()}</ul>
                        </div>
                    }
                })}
            {(!c.check_question.is_empty())
                .then(|| {
                    view! {
                        <div class="check-question">
                            <h4>"Think about it"</h4>
                            <p>{c.check_question}</p>
                        </div>
                    }
                })}
            {(!c.caveat.is_empty())
                .then(|| view! { <p class="caveat"><strong>"Worth checking: "</strong>{c.caveat}</p> })}
            <p class="item-meta">
                {activity.reason}
                " Written by AI, so check it against your material."
            </p>
        </article>
    }
}
