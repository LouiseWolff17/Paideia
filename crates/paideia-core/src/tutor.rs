//! Learning sessions and the tutor's structured content (design doc §4, §5, §7).
//!
//! A session follows one concept through the learning stages. Each time the
//! learner asks the tutor for help, the reply is stored as an [`Activity`].
//!
//! The AI never writes free-form text straight to the screen: its reply must
//! fit [`TutorContent`], and [`TutorContent::validated`] checks it before it is
//! saved or shown (design doc §6, "Validate AI responses").

use std::fmt;

use crate::{
    catalog::{required_text, Timestamp},
    ConceptId, LearningStage, MaterialId, SubjectId, TeachingStrategy, ValidationError,
};

crate::catalog::id_type!(
    /// Stable identifier of a [`LearningSession`].
    SessionId
);
crate::catalog::id_type!(
    /// Stable identifier of an [`Activity`].
    ActivityId
);

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

/// How much of the learner's shared material goes into one tutor request.
/// Anything beyond this is left out, and the interface says so before the
/// session starts (design doc §8: send only the context the request needs).
pub const MATERIAL_PROMPT_BUDGET_CHARS: usize = 40_000;
/// A follow-up question the learner types in.
pub const FOLLOW_UP_MAX_CHARS: usize = 1_000;
/// Upper bounds for AI output. Generous, but they stop a runaway reply.
pub const CONTENT_TITLE_MAX_CHARS: usize = 160;
pub const CONTENT_PARAGRAPH_MAX_CHARS: usize = 2_500;
pub const CONTENT_MAX_PARAGRAPHS: usize = 8;
pub const CONTENT_ITEM_MAX_CHARS: usize = 600;
pub const CONTENT_MAX_STEPS: usize = 10;
pub const CONTENT_MAX_KEY_POINTS: usize = 6;

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

/// A learning session about one concept.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LearningSession {
    pub id: SessionId,
    pub concept_id: ConceptId,
    pub stage: LearningStage,
    /// The material the learner agreed to share with the tutor in this session.
    pub material_ids: Vec<MaterialId>,
    pub activity_count: u32,
    pub created_at: Timestamp,
    /// Last time anything happened in the session (a stage change or new activity).
    pub updated_at: Timestamp,
}

/// A session together with the names needed to show it in a list
/// ("what was last studied", design doc §10).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionSummary {
    pub session: LearningSession,
    pub concept_name: String,
    pub subject_id: SubjectId,
    pub subject_name: String,
}

/// Everything the session page needs in one request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionDetail {
    pub summary: SessionSummary,
    /// Oldest first.
    pub activities: Vec<Activity>,
}

/// Body of `POST /api/concepts/{id}/sessions`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewSession {
    /// Material to share with the tutor. Only material belonging to the concept
    /// is accepted; anything else is ignored.
    #[cfg_attr(feature = "serde", serde(default))]
    pub material_ids: Vec<MaterialId>,
}

/// Body of `PUT /api/sessions/{id}/stage`. Always the learner's own choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StageChange {
    pub stage: LearningStage,
}

// ---------------------------------------------------------------------------
// Activities
// ---------------------------------------------------------------------------

/// One reply from the tutor, with what it was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Activity {
    pub id: ActivityId,
    pub session_id: SessionId,
    /// The stage the session was at when this was generated.
    pub stage: LearningStage,
    /// The teaching approach used, if one was chosen.
    pub strategy: Option<TeachingStrategy>,
    /// Why this approach was used, in plain language (design doc §5: explainable).
    pub reason: String,
    /// The learner's own question, when they asked one.
    pub follow_up: Option<String>,
    pub content: TutorContent,
    pub created_at: Timestamp,
}

/// Body of `POST /api/sessions/{id}/activities`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityRequest {
    /// Set when the learner picked a different approach
    /// ("this doesn't make sense yet").
    #[cfg_attr(feature = "serde", serde(default))]
    pub strategy: Option<TeachingStrategy>,
    /// Set when the learner asked their own question.
    #[cfg_attr(feature = "serde", serde(default))]
    pub follow_up: Option<String>,
}

impl ActivityRequest {
    /// Trim the follow-up question; a blank one counts as no question.
    pub fn validated(&self) -> Result<Self, ValidationError> {
        let follow_up = match self.follow_up.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(q) => Some(required_text(q, "follow_up", "question", FOLLOW_UP_MAX_CHARS)?),
        };
        Ok(Self { strategy: self.strategy, follow_up })
    }
}

/// What the tutor will do for a request, decided by deterministic rules
/// before any AI call is made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorPlan {
    pub stage: LearningStage,
    pub strategy: Option<TeachingStrategy>,
    pub reason: String,
    pub follow_up: Option<String>,
}

impl TutorPlan {
    /// Decide the teaching approach for the next activity.
    ///
    /// * A strategy the learner picked always wins (PREF-06: the learner can
    ///   override automatic teaching decisions).
    /// * A follow-up question is answered directly, keeping the stage's approach.
    /// * Otherwise each stage has a sensible default.
    pub fn choose(stage: LearningStage, request: &ActivityRequest) -> Self {
        let (strategy, reason) = match (request.strategy, &request.follow_up) {
            (Some(s), _) => (Some(s), format!("You asked for this: {}.", s.label().to_lowercase())),
            (None, Some(_)) => (stage.default_strategy(), "Answering your question.".to_owned()),
            (None, None) => (stage.default_strategy(), stage.default_reason().to_owned()),
        };
        Self { stage, strategy, reason, follow_up: request.follow_up.clone() }
    }
}

impl LearningStage {
    /// The approach the tutor uses at this stage unless the learner asks for another.
    pub fn default_strategy(self) -> Option<TeachingStrategy> {
        match self {
            LearningStage::Introduction => None,
            LearningStage::ConcreteExample => Some(TeachingStrategy::ConcreteExample),
            LearningStage::ConceptExplanation => None,
            LearningStage::Formalisation => None,
            LearningStage::GuidedPractice => Some(TeachingStrategy::StepByStep),
            LearningStage::IndependentPractice => None,
            LearningStage::Reflection => Some(TeachingStrategy::SocraticQuestioning),
        }
    }

    fn default_reason(self) -> &'static str {
        match self {
            LearningStage::Introduction => "Starting with what this is and why it matters.",
            LearningStage::ConcreteExample => "A specific example first, before any theory.",
            LearningStage::ConceptExplanation => "The underlying idea, in plain language.",
            LearningStage::Formalisation => "Now the precise terms and rules.",
            LearningStage::GuidedPractice => "Working through a problem together, one step at a time.",
            LearningStage::IndependentPractice => "A new problem to try on your own.",
            LearningStage::Reflection => "Some questions to help you take stock.",
        }
    }

    /// The value stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            LearningStage::Introduction => "introduction",
            LearningStage::ConcreteExample => "concrete_example",
            LearningStage::ConceptExplanation => "concept_explanation",
            LearningStage::Formalisation => "formalisation",
            LearningStage::GuidedPractice => "guided_practice",
            LearningStage::IndependentPractice => "independent_practice",
            LearningStage::Reflection => "reflection",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|stage| stage.as_str() == s)
    }
}

impl TeachingStrategy {
    /// The value stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            TeachingStrategy::ConcreteExample => "concrete_example",
            TeachingStrategy::Analogy => "analogy",
            TeachingStrategy::Visualisation => "visualisation",
            TeachingStrategy::StepByStep => "step_by_step",
            TeachingStrategy::WorkedExample => "worked_example",
            TeachingStrategy::Counterexample => "counterexample",
            TeachingStrategy::Simplification => "simplification",
            TeachingStrategy::PrerequisiteReview => "prerequisite_review",
            TeachingStrategy::SocraticQuestioning => "socratic_questioning",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

// ---------------------------------------------------------------------------
// Tutor content
// ---------------------------------------------------------------------------

/// The shape every tutor reply must have. Empty strings and lists mean "none".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TutorContent {
    /// A short heading.
    pub title: String,
    /// The explanation itself, one idea per paragraph.
    pub paragraphs: Vec<String>,
    /// Ordered steps, for step-by-step explanations and worked examples.
    #[cfg_attr(feature = "serde", serde(default))]
    pub steps: Vec<String>,
    /// A few short take-aways.
    #[cfg_attr(feature = "serde", serde(default))]
    pub key_points: Vec<String>,
    /// One question for the learner to think about.
    #[cfg_attr(feature = "serde", serde(default))]
    pub check_question: String,
    /// Where an analogy stops working, or anything the learner should verify,
    /// e.g. when the explanation goes beyond their material.
    #[cfg_attr(feature = "serde", serde(default))]
    pub caveat: String,
}

/// Why a tutor reply was rejected. Never shown to the learner verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentError(pub String);

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tutor reply rejected: {}", self.0)
    }
}

impl std::error::Error for ContentError {}

fn clean(s: &str, what: &str, max: usize) -> Result<String, ContentError> {
    let s = s.trim();
    let len = s.chars().count();
    if len > max {
        return Err(ContentError(format!("{what} is {len} characters (max {max})")));
    }
    Ok(s.to_owned())
}

fn clean_list(
    items: &[String],
    what: &str,
    max_items: usize,
    max_chars: usize,
) -> Result<Vec<String>, ContentError> {
    let items = items
        .iter()
        .map(|s| clean(s, what, max_chars))
        .filter(|r| !matches!(r, Ok(s) if s.is_empty()))
        .collect::<Result<Vec<_>, _>>()?;
    if items.len() > max_items {
        return Err(ContentError(format!("{} {what} items (max {max_items})", items.len())));
    }
    Ok(items)
}

impl TutorContent {
    /// Trim everything, drop empty items, and reject replies that are empty or
    /// far too long. Run on every AI reply before it is saved or shown.
    pub fn validated(&self) -> Result<Self, ContentError> {
        let title = clean(&self.title, "title", CONTENT_TITLE_MAX_CHARS)?;
        if title.is_empty() {
            return Err(ContentError("missing title".into()));
        }
        let paragraphs =
            clean_list(&self.paragraphs, "paragraph", CONTENT_MAX_PARAGRAPHS, CONTENT_PARAGRAPH_MAX_CHARS)?;
        let steps = clean_list(&self.steps, "step", CONTENT_MAX_STEPS, CONTENT_ITEM_MAX_CHARS)?;
        if paragraphs.is_empty() && steps.is_empty() {
            return Err(ContentError("no paragraphs or steps".into()));
        }
        Ok(Self {
            title,
            paragraphs,
            steps,
            key_points: clean_list(&self.key_points, "key point", CONTENT_MAX_KEY_POINTS, CONTENT_ITEM_MAX_CHARS)?,
            check_question: clean(&self.check_question, "check question", CONTENT_ITEM_MAX_CHARS)?,
            caveat: clean(&self.caveat, "caveat", CONTENT_ITEM_MAX_CHARS)?,
        })
    }
}

// ---------------------------------------------------------------------------
// AI settings shared with the browser
// ---------------------------------------------------------------------------

/// Response of `GET /api/ai`, and body of `PUT /api/ai/consent`'s reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AiStatus {
    /// An Anthropic API key is set on the server.
    pub configured: bool,
    /// The learner has read how AI requests work and agreed to them.
    pub consented: bool,
}

/// Body of `PUT /api/ai/consent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsentChange {
    pub consented: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> TutorContent {
        TutorContent {
            title: " Eigenvalues ".into(),
            paragraphs: vec!["  A vector that only stretches. ".into(), "   ".into()],
            steps: vec![],
            key_points: vec!["".into(), "Direction stays the same".into()],
            check_question: " What happens to its length? ".into(),
            caveat: String::new(),
        }
    }

    #[test]
    fn content_is_trimmed_and_empty_items_dropped() {
        let v = content().validated().unwrap();
        assert_eq!(v.title, "Eigenvalues");
        assert_eq!(v.paragraphs, vec!["A vector that only stretches."]);
        assert_eq!(v.key_points, vec!["Direction stays the same"]);
        assert_eq!(v.check_question, "What happens to its length?");
    }

    #[test]
    fn content_needs_a_title_and_a_body() {
        let mut c = content();
        c.title = "  ".into();
        assert!(c.validated().is_err());

        let mut c = content();
        c.paragraphs = vec![" ".into()];
        assert!(c.validated().is_err());

        // Steps alone are enough.
        c.steps = vec!["First".into()];
        assert!(c.validated().is_ok());
    }

    #[test]
    fn runaway_content_is_rejected() {
        let mut c = content();
        c.paragraphs = vec!["x".into(); CONTENT_MAX_PARAGRAPHS + 1];
        assert!(c.validated().is_err());

        let mut c = content();
        c.title = "t".repeat(CONTENT_TITLE_MAX_CHARS + 1);
        assert!(c.validated().is_err());
    }

    #[test]
    fn learner_choice_overrides_the_stage_default() {
        let req = ActivityRequest { strategy: Some(TeachingStrategy::Analogy), follow_up: None };
        let plan = TutorPlan::choose(LearningStage::ConcreteExample, &req);
        assert_eq!(plan.strategy, Some(TeachingStrategy::Analogy));
        assert!(plan.reason.contains("an analogy"));

        let plan = TutorPlan::choose(LearningStage::ConcreteExample, &ActivityRequest::default());
        assert_eq!(plan.strategy, Some(TeachingStrategy::ConcreteExample));
    }

    #[test]
    fn follow_up_questions_are_trimmed_and_blank_ones_ignored() {
        let req = ActivityRequest { strategy: None, follow_up: Some("  why?  ".into()) };
        assert_eq!(req.validated().unwrap().follow_up.as_deref(), Some("why?"));

        let req = ActivityRequest { strategy: None, follow_up: Some("   ".into()) };
        assert_eq!(req.validated().unwrap().follow_up, None);

        let req = ActivityRequest { strategy: None, follow_up: Some("x".repeat(FOLLOW_UP_MAX_CHARS + 1)) };
        assert_eq!(req.validated().unwrap_err().field, "follow_up");
    }

    #[test]
    fn stages_and_strategies_round_trip_through_strings() {
        for s in LearningStage::ALL {
            assert_eq!(LearningStage::parse(s.as_str()), Some(s));
        }
        for t in TeachingStrategy::ALL {
            assert_eq!(TeachingStrategy::parse(t.as_str()), Some(t));
        }
        assert_eq!(LearningStage::parse("nope"), None);
    }
}
