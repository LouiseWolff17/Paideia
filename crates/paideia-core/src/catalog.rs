//! The learner's own catalogue: subjects, the concepts inside them, and the
//! learning material attached to each concept (design doc §7, "Initial entities").
//!
//! Validation lives here so the web interface and the server apply exactly the
//! same rules.

use std::fmt;

/// Milliseconds since the Unix epoch (UTC).
pub type Timestamp = i64;

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $name(pub i64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::num::ParseIntError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map($name)
            }
        }
    };
}

id_type!(
    /// Stable identifier of a [`Subject`]. Never reused after deletion.
    SubjectId
);
id_type!(
    /// Stable identifier of a [`Concept`]. Never reused after deletion.
    ConceptId
);
id_type!(
    /// Stable identifier of a [`Material`]. Never reused after deletion.
    MaterialId
);

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

pub const NAME_MAX_CHARS: usize = 120;
pub const DESCRIPTION_MAX_CHARS: usize = 2_000;
pub const MATERIAL_TITLE_MAX_CHARS: usize = 200;
pub const MATERIAL_SOURCE_MAX_CHARS: usize = 300;
/// Roughly 30–40 pages of plain text. Longer material can be split up.
pub const MATERIAL_BODY_MAX_CHARS: usize = 200_000;

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// A university course or broader area of study.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Subject {
    pub id: SubjectId,
    pub name: String,
    pub description: String,
    pub concept_count: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A specific idea or skill within a subject.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Concept {
    pub id: ConceptId,
    pub subject_id: SubjectId,
    pub name: String,
    pub summary: String,
    pub material_count: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Where a piece of material came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum MaterialKind {
    /// Typed or pasted in by the learner.
    ManualText,
    // PDF and slide imports arrive in milestone 6.
}

impl MaterialKind {
    /// The value stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            MaterialKind::ManualText => "manual_text",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "manual_text" => Some(MaterialKind::ManualText),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MaterialKind::ManualText => "Your text",
        }
    }
}

/// Source material attached to a concept. Kept separate from anything the AI
/// generates later (design doc §7, persistence rules).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Material {
    pub id: MaterialId,
    pub concept_id: ConceptId,
    pub kind: MaterialKind,
    pub title: String,
    /// Optional note on where it came from, e.g. "Lecture 3 slides".
    pub source: String,
    pub body: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Inputs (what the learner submits when creating or editing)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubjectInput {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConceptInput {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summary: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialInput {
    pub title: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub source: String,
    pub body: String,
}

/// Why an input was not accepted, phrased for the learner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// The input field the problem is about, e.g. `"name"`.
    pub field: &'static str,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ValidationError {}

fn required(
    value: &str,
    field: &'static str,
    what: &str,
    max: usize,
) -> Result<String, ValidationError> {
    let v = value.trim();
    if v.is_empty() {
        return Err(ValidationError { field, message: format!("Please give it a {what}.") });
    }
    optional(v, field, what, max)
}

fn optional(
    value: &str,
    field: &'static str,
    what: &str,
    max: usize,
) -> Result<String, ValidationError> {
    let v = value.trim();
    let len = v.chars().count();
    if len > max {
        return Err(ValidationError {
            field,
            message: format!(
                "The {what} is a bit long ({len} characters). Please keep it under {max}."
            ),
        });
    }
    Ok(v.to_owned())
}

impl SubjectInput {
    /// Trim whitespace and check limits.
    pub fn validated(&self) -> Result<Self, ValidationError> {
        Ok(Self {
            name: required(&self.name, "name", "name", NAME_MAX_CHARS)?,
            description: optional(&self.description, "description", "description", DESCRIPTION_MAX_CHARS)?,
        })
    }
}

impl ConceptInput {
    /// Trim whitespace and check limits.
    pub fn validated(&self) -> Result<Self, ValidationError> {
        Ok(Self {
            name: required(&self.name, "name", "name", NAME_MAX_CHARS)?,
            summary: optional(&self.summary, "summary", "summary", DESCRIPTION_MAX_CHARS)?,
        })
    }
}

impl MaterialInput {
    /// Trim the title and source, and check limits. The body keeps its inner
    /// formatting; only leading and trailing blank space is removed.
    pub fn validated(&self) -> Result<Self, ValidationError> {
        let body = self.body.trim();
        if body.is_empty() {
            return Err(ValidationError {
                field: "body",
                message: "Please add some text.".into(),
            });
        }
        Ok(Self {
            title: required(&self.title, "title", "title", MATERIAL_TITLE_MAX_CHARS)?,
            source: optional(&self.source, "source", "source", MATERIAL_SOURCE_MAX_CHARS)?,
            body: optional(body, "body", "text", MATERIAL_BODY_MAX_CHARS)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_input_is_trimmed() {
        let input = SubjectInput { name: "  Linear algebra \n".into(), description: "  ".into() };
        assert_eq!(
            input.validated(),
            Ok(SubjectInput { name: "Linear algebra".into(), description: String::new() })
        );
    }

    #[test]
    fn blank_names_are_rejected() {
        let err = SubjectInput { name: "   ".into(), ..Default::default() }.validated().unwrap_err();
        assert_eq!(err.field, "name");
        let err = ConceptInput { name: String::new(), ..Default::default() }.validated().unwrap_err();
        assert_eq!(err.field, "name");
    }

    #[test]
    fn limits_count_characters_not_bytes() {
        // 120 multi-byte characters is fine; 121 is not.
        let ok = "é".repeat(NAME_MAX_CHARS);
        assert!(SubjectInput { name: ok.clone(), ..Default::default() }.validated().is_ok());
        let too_long = format!("{ok}é");
        let err = SubjectInput { name: too_long, ..Default::default() }.validated().unwrap_err();
        assert_eq!(err.field, "name");
    }

    #[test]
    fn material_keeps_inner_formatting() {
        let input = MaterialInput {
            title: " Notes ".into(),
            source: String::new(),
            body: "\n  line one\n\n    indented\n".into(),
        };
        let v = input.validated().unwrap();
        assert_eq!(v.title, "Notes");
        assert_eq!(v.body, "line one\n\n    indented");
    }

    #[test]
    fn material_needs_title_and_body() {
        let no_body = MaterialInput { title: "T".into(), source: String::new(), body: " \n ".into() };
        assert_eq!(no_body.validated().unwrap_err().field, "body");
        let no_title = MaterialInput { title: String::new(), source: String::new(), body: "x".into() };
        assert_eq!(no_title.validated().unwrap_err().field, "title");
    }

    #[test]
    fn ids_round_trip_through_strings() {
        let id: SubjectId = "42".parse().unwrap();
        assert_eq!(id, SubjectId(42));
        assert_eq!(id.to_string(), "42");
        assert!("abc".parse::<ConceptId>().is_err());
    }

    #[test]
    fn material_kind_round_trips() {
        assert_eq!(MaterialKind::parse(MaterialKind::ManualText.as_str()), Some(MaterialKind::ManualText));
        assert_eq!(MaterialKind::parse("pdf"), None);
    }
}
