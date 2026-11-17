# Paideia

A calm, local-first learning space that helps a university student understand
difficult ideas, remember them, and turn big tasks into small next steps.

This is **Milestone 1: Foundation**. It has a Rust workspace, a Leptos web
interface with the six main areas, a local Axum server with a health check,
and the shared domain types (learning stages, teaching strategies, evidence
dimensions).

## Layout

```
paideia/
├── Cargo.toml            workspace
├── DESIGN.md             project design and roadmap
├── .env.example          copy to .env (never commit .env)
└── crates/
    ├── paideia-core/     domain types and deterministic rules (no UI/AI/DB code)
    ├── paideia-server/   Axum server: /api, and serves the built interface
    └── paideia-web/      Leptos frontend, compiled to WebAssembly with Trunk
```

## One-time setup

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cp .env.example .env        # add your Anthropic key later (Milestone 3)
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
cargo test -p paideia-core
```

## Privacy notes

* The server listens on `127.0.0.1` only, unless you change `PAIDEIA_BIND`.
* The Anthropic key is read on the server and is never sent to the browser.
  `/api/health` only reports *whether* a key is set.
* Configuration is never logged with the key; prompts and notes are never logged.

## Roadmap

The full design (goals, learning model, data model, requirements) is in
[DESIGN.md](DESIGN.md).

1. **Foundation** ← you are here
2. Core learning data: subjects, concepts, SQLite, manual text material
3. First tutoring session: Anthropic adapter, structured explanations
4. Practice and adaptation: exercises, attempts, strategy selection
5. Remember and progress: review scheduling, evidence, history
6. Polish and import: accessibility pass, export/delete, PDF/slides
