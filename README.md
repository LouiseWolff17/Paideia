# Paideia

A calm, local-first learning space that helps a university student understand
difficult ideas, remember them, and turn big tasks into small next steps.

This is **Milestone 3: First tutoring session**. On top of subjects, concepts
and your own text material (Milestone 2), you can now start a learning
session on a concept, choose which of your material the tutor may use, and
work through the stages with an AI tutor (Anthropic's Claude). The tutor gives
structured explanations, examples and questions; you can ask your own
questions or say "this doesn't make sense yet" and pick a different approach.
Sessions and the tutor's replies are saved locally, so you can resume later.

## Layout

```
paideia/
├── Cargo.toml            workspace
├── DESIGN.md             project design and roadmap
├── .env.example          copy to .env (never commit .env)
└── crates/
    ├── paideia-core/     domain types and deterministic rules (no UI/AI/DB code)
    ├── paideia-server/   Axum server: /api, SQLite storage, serves the built interface
    │   └── migrations/   database schema, applied automatically at startup
    └── paideia-web/      Leptos frontend, compiled to WebAssembly with Trunk
```

## One-time setup

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cp .env.example .env        # then add your Anthropic API key to use the tutor
```

## Running during development

Use two terminals from the project root:

```sh
# 1. Local server (API on http://127.0.0.1:3000)
cargo run -p paideia-server

# 2. Web interface with live reload (http://127.0.0.1:8080)
cd crates/paideia-web && trunk serve --open
```

Trunk forwards `/api/` requests to the server, so the browser only ever talks
to your own machine.

## Running as one app

```sh
cd crates/paideia-web && trunk build --release && cd ../..
cargo run --release -p paideia-server
# open http://127.0.0.1:3000
```

## Tests

```sh
cargo test
```

Server tests use a private in-memory database, so they never touch your data.

## Your data

Everything is stored in `data/paideia.db` (or in `PAIDEIA_DATA_DIR`). To back
up, stop the server and copy the whole `data` folder. To start fresh, stop the
server and delete it.

## The AI tutor

Set `ANTHROPIC_API_KEY` in `.env` and restart the server. Optional settings:
`ANTHROPIC_MODEL` (default `claude-opus-5-5`) and `ANTHROPIC_EFFORT`
(`low` … `max`, default `medium`).

Before the first request, Paideia explains what is sent and asks you to agree
(you can withdraw in Preferences). Each request contains only the concept, the
material you ticked when starting that session, your question if you asked
one, and the titles of earlier replies in the session. Replies must match a
fixed JSON structure and are checked before they are saved or shown.

Requests use structured outputs and Anthropic's server-side refusal fallback
(`fallbacks: "default"`), so a declined request is retried on another model in
the same call. Without a key, everything except the tutor's replies still works.

## Privacy notes

* The server listens on `127.0.0.1` only, unless you change `PAIDEIA_BIND`.
* The Anthropic key is read on the server and is never sent to the browser.
  `/api/health` only reports *whether* a key is set.
* Configuration is never logged with the key; prompts, notes and tutor replies are never logged.
* Nothing is sent to Anthropic until you have agreed and press a button that asks the tutor.

## Roadmap

The full design (goals, learning model, data model, requirements) is in
[DESIGN.md](DESIGN.md).

1. Foundation ✓
2. Core learning data ✓
3. **First tutoring session** ← you are here: Anthropic adapter, structured explanations
4. Practice and adaptation: exercises, attempts, strategy selection
5. Remember and progress: review scheduling, evidence, history
6. Polish and import: accessibility pass, export/delete, PDF/slides
