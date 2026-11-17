# Paideia: Project Design

*A personal, adaptive learning space.* Version 1.0, 17 November 2026.

> A calm, locally stored learning environment that helps a university student
> understand difficult ideas, remember what they learn, and turn overwhelming
> work into manageable next steps.

The detailed requirement IDs in [Appendix A](#appendix-a-detailed-requirements-from-sdd-v01)
come from the earlier SDD v0.1. Where the two documents disagree (for example on
milestone order), this document wins.

## Contents

1. [Overview and product vision](#1-overview-and-product-vision)
2. [Goals, audience and design principles](#2-goals-audience-and-design-principles)
3. [Scope and requirements](#3-scope-and-requirements)
4. [Experience and learning model](#4-experience-and-learning-model)
5. [Adaptive tutoring system](#5-adaptive-tutoring-system)
6. [Technical architecture](#6-technical-architecture)
7. [Data model and persistence](#7-data-model-and-persistence)
8. [Privacy, safety and accessibility](#8-privacy-safety-and-accessibility)
9. [Development roadmap](#9-development-roadmap)
10. [MVP definition and evaluation](#10-mvp-definition-and-evaluation)

---

## 1. Overview and product vision

Paideia is a personal web application for university learning. It combines
structured learning activities, adaptive explanations, practice and review in
one calm workspace. Its purpose is not simply to present information, but to
help the learner build a usable understanding of it and return to it over time.

The application adapts how it teaches based on evidence from the learner's
responses. When an explanation does not work, Paideia should try a different
route: a concrete example, an analogy, a visual explanation, a smaller step, a
prerequisite review or a guided question.

The first version is designed for one learner on a personal computer, without
accounts. Learning data and imported files are stored locally. The AI provider
is Anthropic; content sent to its API is processed outside the local device.

## 2. Goals, audience and design principles

### Target audience

The initial audience is the project owner: a university student who wants
support with abstract concepts, remembering information, getting started on
tasks and maintaining concentration. The design should be useful to other
university students later, but the first release is intentionally personal
rather than multi-user.

### Product goals

- Improve conceptual understanding by connecting abstract ideas to concrete examples.
- Support long-term retention through active recall and spaced review.
- Reduce the friction of getting started by turning broad tasks into small, clear actions.
- Help the learner regain focus without punishment, streak pressure or distracting interfaces.
- Adapt explanations and practice to demonstrated understanding, not assumptions about ability.
- Support growing independence rather than completing coursework on the learner's behalf.

### Design principles

- **Clarity before density:** show one meaningful next step at a time.
- **Concrete before abstract:** begin with examples where appropriate, then introduce formal definitions.
- **Flexible pacing:** allow pauses, review and backward movement without penalty.
- **Evidence over labels:** track what the learner can do, not a single judgement of intelligence or ability.
- **Local-first by default:** keep personal learning history and source files on the device.
- **Support autonomy:** explain, scaffold and coach; do not silently take over the learner's work.

## 3. Scope and requirements

| Area | Requirement | Priority |
|---|---|---|
| Subjects & concepts | Create subjects, organise concepts and track progress through them. | MVP |
| Learning material | Add text manually; later import PDF and slide content. | Manual: MVP · Files: later |
| AI tutor | Generate explanations, examples, questions and exercises grounded in supplied material. | MVP |
| Practice | Support guided exercises, independent attempts and feedback. | MVP |
| Adaptation | Change teaching strategy based on learner responses and feedback. | Basic: MVP |
| Memory | Save learning history and provide review prompts. | Basic: MVP |
| Task support | Break a broad study task into small actionable steps. | Later |
| Progress | Show concept-level evidence and review history. | Basic: MVP |
| Preferences | Adjust explanation style, session length and interface preferences. | Basic: MVP |

### Non-goals

- Diagnosing or treating ADHD, AuDHD or any other medical condition.
- Automatically completing assignments or replacing the learner's own reasoning.
- Enforcing fixed study schedules, streaks, leaderboards or competitive rankings.
- Building a multi-user social platform in the first release.
- Supporting every file format or external learning platform from day one.

## 4. Experience and learning model

The interface is organised around six main areas. Each has a clear purpose and
avoids duplicating controls.

| Module | Purpose |
|---|---|
| Learn | Continue a learning session, study a concept or start a new session. |
| Remember | Review concepts using active recall and spaced repetition. |
| Explore | Browse subjects, concepts and imported learning materials. |
| My Tasks | Turn study intentions into small next actions and resume unfinished work. |
| Progress | See learning evidence, recurring difficulties and concepts ready for review. |
| Preferences | Control presentation, learning preferences, privacy and AI settings. |

### A learning session

A session follows a flexible progression. The learner can move forward when
ready, repeat a stage, or return to an earlier explanation. The sequence is not
a rigid lesson template: a familiar topic may need less scaffolding, while a
difficult topic may need several concrete examples.

```
Introduce → Make concrete → Explain → Formalise → Practise → Reflect
(orient)    (example or     (build     (notation   (guided →   (check
            analogy)        concept)   and rules)  independent) understanding)
```

Evidence from each attempt informs the next explanation, activity and review.
The learner can move backwards whenever a prerequisite or earlier idea needs
attention.

### Learning stages

- **Introduction:** establish the topic, its purpose and what the learner is trying to understand.
- **Concrete example:** connect the idea to a familiar situation or worked example.
- **Concept explanation:** explain the underlying idea in plain language.
- **Formalisation:** introduce precise terminology, notation, rules or definitions.
- **Guided practice:** solve a problem together, revealing one step at a time.
- **Independent practice:** let the learner attempt a new problem without step-by-step help.
- **Reflection:** identify what is understood, what remains uncertain and what to revisit.

The stages form a deterministic state machine owned by the application. The AI
may *recommend* a transition, but the application validates it and keeps the
authoritative state.

The learner always has a way to say "I don't understand this" and get an
alternative approach (another example, simpler wording, smaller steps, a
visual, an analogy, or a prerequisite review) without having to explain why.

## 5. Adaptive tutoring system

Paideia adapts its teaching method using observable evidence. A wrong answer
alone is not enough to conclude that the learner does not understand a topic.
The system considers the type of task, the learner's explanation, repeated
errors, confidence and whether the learner can apply the concept in a new
context.

### Teaching strategies

- **Concrete example:** show a specific instance before generalising.
- **Analogy:** relate an unfamiliar concept to a familiar one, while stating where the analogy stops working.
- **Visualisation:** use a diagram, spatial representation or sequence.
- **Step-by-step:** break a complex explanation or solution into smaller parts.
- **Worked example:** demonstrate a complete solution and explain why each step is taken.
- **Counterexample:** show a case that does not satisfy a definition or rule.
- **Simplification:** reduce the number of ideas introduced at once.
- **Prerequisite review:** revisit an earlier concept that the current task depends on.
- **Socratic questioning:** ask a focused question that helps the learner reason independently.

### The adaptive loop

```
Learner action → Evidence → Learner model
                                  ↓
Next activity ← Choose strategy ← Identify need
      └──────────── repeat ─────────────┘
```

Repeat as the learner responds; retain evidence over time.

### Learner model

The learner model is a structured record of evidence, not a permanent label.
It tracks distinct dimensions, because success in one does not guarantee
success in another:

- **Recognition:** can the learner identify a concept or correct example?
- **Explanation:** can the learner explain the idea in their own words?
- **Recall:** can the learner retrieve it without seeing the answer?
- **Application:** can the learner use it in a familiar problem?
- **Transfer:** can the learner apply it in a new or unfamiliar context?

The system retains timestamps, activity type, correctness, hints used and
optional self-reported confidence. It distinguishes *insufficient evidence*
from *demonstrated difficulty*, and avoids reducing progress to one opaque
mastery score. Recommendations are explainable, for example: "Let's review the
prerequisite because the last two exercises depended on it."

## 6. Technical architecture

The first implementation is a local modular monolith. The browser talks to a
local Rust server; the server stores data locally and calls Anthropic when an
AI response is requested.

```
          Learner's browser (Leptos + WebAssembly)
                          │
          Local application server (Axum + Tokio)
           │              │                 │
  Domain & application   Local persistence   AI integration
  (learning rules,       (SQLite,            (Anthropic API adapter)
   progress, tasks)       imported files)          │
                                              leaves the device
```

### Technology choices

| Layer | Technology | Responsibility |
|---|---|---|
| Frontend | Leptos 0.8 + Rust/WASM (Trunk) | Interactive web interface and learning experience. |
| Local backend | Axum + Tokio | HTTP API, orchestration and local application services. |
| Domain | Shared Rust crate | Core types, learning stages and deterministic rules. |
| Database | SQLite + SQLx | Local subjects, concepts, sessions, evidence and preferences. |
| AI | Anthropic API via Reqwest | Generate structured tutoring content; API key remains server-side. |
| Files | Local filesystem | Store imported source documents and extracted text. |

### Architectural boundaries

- Keep learning rules, progress calculations and scheduling deterministic and testable outside the AI layer.
- Treat the AI provider as an adapter so prompts and provider-specific request formats do not leak into domain logic.
- Validate AI responses before saving or displaying them; prefer structured response schemas.
- Ground explanations in the learner's selected concept and supplied materials where available.
- Keep the initial deployment local; do not introduce microservices before there is a demonstrated need.

### Repository structure

```
paideia/
├── Cargo.toml
├── DESIGN.md
├── .env.example
└── crates/
    ├── paideia-core/     # domain types and rules
    ├── paideia-server/   # Axum API and application wiring
    └── paideia-web/      # Leptos frontend
```

This is the intentionally small starting structure. Additional crates for
application services, AI and infrastructure can be extracted when their
boundaries become useful; they are not a prerequisite for the MVP.

## 7. Data model and persistence

The following is a conceptual map rather than a final SQL schema.

```
Subject → Concept → Material
             ↓
     Learning session → Evidence → Review
```

### Initial entities

- **Subject:** a university course or broader area of study.
- **Concept:** a specific idea or skill within a subject.
- **Material:** an imported document, slide deck or manually entered text, with source metadata.
- **LearningSession:** a session associated with a concept, including its current stage and timestamps.
- **Activity:** an explanation, question, exercise or review prompt shown during a session.
- **Attempt:** the learner's response, feedback, hints and outcome.
- **LearningEvidence:** evidence mapped to recognition, explanation, recall, application or transfer.
- **ReviewSchedule:** when a concept should next be revisited, based on recall evidence.
- **Preference:** learner-controlled settings for presentation and tutoring behaviour.

### Persistence rules

- Store structured learning records in SQLite and original imported files in a local data directory.
- Use stable IDs and timestamps so sessions can be resumed and evidence can be reviewed.
- Keep raw source material separate from AI-generated explanations and learner responses.
- Provide a clear way to export or delete local learning data.
- Back up the database and document directory together; a database backup without source files may be incomplete.

## 8. Privacy, safety and accessibility

### Local-first privacy

No user account is required for the initial version. Learning history and
imported files are stored locally. However, using Anthropic means that the text
included in a request (potentially including excerpts from learning materials)
is sent to an external service for processing. The interface must communicate
this clearly before the first AI request and provide a way to use non-AI
features without sending content. The app must not claim that all processing
is local.

### Security requirements

- Keep the Anthropic API key in a local server environment variable; never expose it in browser code or commit it to Git.
- Do not log API keys, full prompts, private notes or complete document contents by default.
- Limit the local server to loopback access unless the user explicitly configures otherwise.
- Validate file type and size, and treat imported documents and AI output as untrusted input.
- Use explicit user actions before sending selected material to the AI provider.
- Send only the context the current request needs, never the whole learning history.
- Make data deletion and export understandable and reliable.

### Accessibility and cognitive load

- Aim for WCAG 2.2 AA: keyboard access, visible focus, sufficient contrast and semantic structure.
- Use plain language, consistent navigation and predictable layouts.
- Avoid unnecessary animations, notification pressure, streaks and competing visual emphasis.
- Allow the learner to pause, resume and change the level of detail.
- Present long tasks as small steps, with one clear suggested next action.
- Do not assume that one teaching style or attention strategy works for every session.
- No guilt-based notifications or negative messages about inactivity.

### Visual design direction

Calm, warm and gently encouraging rather than childish or overly gamified.

| Colour | Hex | Use |
|---|---|---|
| Soft pink | `#F7E8EC` | Soft surfaces and gentle highlights. |
| Sage green | `#DCE9DC` | Positive, calm supporting surfaces. |
| Warm cream | `#FAF7F0` | Main background. |
| Forest ink | `#454A43` | Primary text. |
| Muted pink | `#C47F98` | Brand accent and selected emphasis. |
| Muted sage | `#8B9D88` | Secondary accent. |

## 9. Development roadmap

| # | Milestone | Scope | Status |
|---|---|---|---|
| 1 | Foundation | Rust workspace, Leptos frontend, Axum server, local configuration, health endpoint, visual shell and navigation. | Done |
| 2 | Core learning data | Subjects, concepts, SQLite persistence, basic create/read/update/delete flows, manual text material. | Done |
| 3 | First tutoring session | Connect the server to Anthropic, a safe provider adapter, structured explanations, examples and questions. | Next |
| 4 | Practice and adaptation | Guided and independent exercises, attempt recording, feedback, basic strategy selection. | |
| 5 | Remember and progress | Review scheduling, evidence dimensions, session history, a simple progress view. | |
| 6 | Polish and import | Accessibility, error handling, export/delete flows, PDF/slide text extraction. | |

### Implementation approach

- Build one complete vertical slice at a time, from interface to persistence to tests.
- Keep the first AI feature narrow: one concept, one explanation and one learner response.
- Add abstractions only when a second real use case demonstrates their value.
- Test deterministic domain logic independently from AI-generated content.
- Treat prompt quality and teaching behaviour as product features that require iteration and evaluation.

## 10. MVP definition and evaluation

The MVP is successful when the learner can create a subject and concept, add
source text, start an AI-assisted learning session, move through an
explanation and practice activity, save the session and return later. The
system should adapt at least one teaching choice based on the learner's
response.

### MVP acceptance criteria

- [x] A subject and its concepts can be created, edited and stored locally.
- [x] A learner can add text manually and associate it with a concept.
- [ ] The learner can request an explanation and a concrete example through the local server.
- [ ] The learner can answer a guided or independent exercise and receive feedback.
- [ ] The session and attempt history persist after the application is closed.
- [ ] A basic adaptation rule can choose a different teaching strategy after evidence of difficulty.
- [ ] The learner can see what was last studied and resume the session.
- [ ] The API key is not present in frontend assets, and the external AI data flow is disclosed.

### How to evaluate Paideia

Evaluation should focus on whether the application helps learning rather than
how long it keeps someone using it. Useful measures: whether the learner can
explain a concept after a session, recall it after a delay, apply it to a new
problem, resume an interrupted task and identify a useful next step. Collect
these transparently and never interpret them as clinical or diagnostic
indicators. Don't treat a teaching strategy as universally effective based on a
few observations.

### Risks and mitigations

| Risk | Mitigation |
|---|---|
| AI gives an incorrect explanation | Ground responses in supplied material where possible; validate structure; invite verification; allow corrections. |
| AI does the learner's work | Use hints, questions and scaffolded feedback; avoid returning complete assignment submissions as the default. |
| Overly complex MVP | Prioritise one end-to-end learning flow before advanced imports, dashboards or integrations. |
| Sensitive data leaves device | Explain the Anthropic boundary; send only selected relevant content; offer non-AI functionality. |
| Progress score misleads | Track multiple evidence dimensions and show the basis for recommendations. |

### Conclusion

Paideia should begin as a small, dependable learning companion: a local
application that helps the learner understand one concept, practise it, see
what they have learned and return to it later. Its defining quality is
thoughtful adaptation, changing the explanation or next step when the current
approach is not helping, while keeping the learner in control of their work
and data.

The most important implementation decision is to build the complete learning
loop before expanding the feature set.

---

## Appendix A: Detailed requirements (from SDD v0.1)

Priorities: **Must** = needed for the first usable app, **Should** = follows
soon after, **Could** = optional. Milestone numbers in this appendix follow the
roadmap above, not the v0.1 ordering.

**Subjects (SUB)**: SUB-01 create subject (Must) · SUB-02 edit subject (Must) ·
SUB-03 delete subject (Must) · SUB-04 create concepts within a subject (Must) ·
SUB-05 group concepts (Should) · SUB-06 show subject progress (Should)

**Material (MAT)**: MAT-01 create concepts manually (Must) · MAT-02 enter or
paste text (Must) · MAT-03 upload PDF (Should) · MAT-04 upload slides, PPTX
first (Should) · MAT-05 extract concepts from material (Should) · MAT-06
identify definitions and examples (Should) · MAT-07 correct extracted content
(Must) · MAT-08 reference original material while teaching (Should)

**AI tutor (AI)**: AI-01 accessible explanations (Must) · AI-02 concrete
examples (Must) · AI-03 analogies (Should) · AI-04 step-by-step explanations
(Must) · AI-05 formal definitions (Must) · AI-06 guided exercises (Must) ·
AI-07 independent exercises (Must) · AI-08 adapt to feedback (Must) · AI-09
spot misconceptions (Should) · AI-10 spot missing prerequisites (Should) ·
AI-11 visual representations (Could) · AI-12 follow-up questions (Must).
The tutor must use structured learner data (attempts, preferences, evidence),
not only conversation history.

**Memory (MEM)**: MEM-01 record completed activities (Must) · MEM-02 store
attempts (Must) · MEM-03 active recall questions (Should) · MEM-04 schedule
reviews (Should) · MEM-05 track delayed recall (Should) · MEM-06 distinguish
recognition from application (Must) · MEM-07 flag concepts needing practice
(Should)

**Tasks and focus (TASK)**: TASK-01 break assignments into steps (Should) ·
TASK-02 recommend next activity (Must) · TASK-03 save unfinished sessions
(Must) · TASK-04 resume sessions (Must) · TASK-05 optional break reminders
(Could) · TASK-06 distraction-minimised interface (Must)

**Progress (PRO)**: PRO-01 learning history (Should) · PRO-02 conceptual
mastery (Should) · PRO-03 review history (Should) · PRO-04 compare strategies
(Could) · PRO-05 areas needing practice (Should)

**Preferences (PREF)**: PREF-01 explanation complexity (Must) · PREF-02
preferred strategies (Must) · PREF-03 visual appearance (Should) · PREF-04
animation preferences (Should) · PREF-05 break preferences (Could) · PREF-06
override automatic teaching decisions (Must)

**AI decision pipeline**: receive attempt or feedback → retrieve relevant
history → update learner model → identify difficulties → select strategy →
generate next activity → present it → collect new evidence. The AI never
decides mastery on conversational confidence alone.
