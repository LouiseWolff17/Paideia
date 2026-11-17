-- Milestone 3: learning sessions and the tutor's replies.
-- AI-generated content lives here, separate from the learner's own material.

-- Small key/value settings, e.g. when the learner agreed to AI requests.
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE learning_sessions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    concept_id  INTEGER NOT NULL REFERENCES concepts (id) ON DELETE CASCADE,
    stage       TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX sessions_by_concept ON learning_sessions (concept_id);
CREATE INDEX sessions_by_recency ON learning_sessions (updated_at DESC);

-- Which material the learner chose to share with the tutor in a session.
CREATE TABLE session_materials (
    session_id  INTEGER NOT NULL REFERENCES learning_sessions (id) ON DELETE CASCADE,
    material_id INTEGER NOT NULL REFERENCES materials (id) ON DELETE CASCADE,
    PRIMARY KEY (session_id, material_id)
);

-- One tutor reply. `content` is validated TutorContent stored as JSON.
CREATE TABLE activities (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  INTEGER NOT NULL REFERENCES learning_sessions (id) ON DELETE CASCADE,
    stage       TEXT    NOT NULL,
    strategy    TEXT,
    reason      TEXT    NOT NULL DEFAULT '',
    follow_up   TEXT,
    content     TEXT    NOT NULL,
    model       TEXT    NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL
);

CREATE INDEX activities_by_session ON activities (session_id);
