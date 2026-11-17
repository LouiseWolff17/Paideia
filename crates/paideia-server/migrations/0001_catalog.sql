-- Milestone 2: subjects, concepts and manually entered material.
-- Timestamps are milliseconds since the Unix epoch (UTC).
-- AUTOINCREMENT guarantees an ID is never reused after deletion, so IDs stay stable.

CREATE TABLE subjects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE concepts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id  INTEGER NOT NULL REFERENCES subjects (id) ON DELETE CASCADE,
    name        TEXT    NOT NULL,
    summary     TEXT    NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX concepts_by_subject ON concepts (subject_id);

-- Raw source material only. AI-generated content will live in its own tables.
CREATE TABLE materials (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    concept_id  INTEGER NOT NULL REFERENCES concepts (id) ON DELETE CASCADE,
    kind        TEXT    NOT NULL CHECK (kind IN ('manual_text')),
    title       TEXT    NOT NULL,
    source      TEXT    NOT NULL DEFAULT '',
    body        TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX materials_by_concept ON materials (concept_id);
