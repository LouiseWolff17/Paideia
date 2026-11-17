//! The learning-session state machine (SDD §5.2).
//!
//! The AI may *recommend* a transition, but this module is the authority on
//! whether it is allowed. The rules are:
//!
//! * Moving **back** to any earlier stage is always allowed, for anyone.
//!   Returning to an earlier idea is never a failure.
//! * Staying on the current stage (repeating it) is always allowed.
//! * Moving **forward by one** stage is always allowed.
//! * **Skipping ahead** several stages is allowed only when the learner chooses it.
//!   The learner can always override the tutor (PREF-06), but the tutor
//!   may not skip scaffolding on its own.

/// One stage of a flexible learning journey (design doc §4, Figure 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum LearningStage {
    Introduction,
    ConcreteExample,
    ConceptExplanation,
    Formalisation,
    GuidedPractice,
    IndependentPractice,
    Reflection,
}

/// Who asked for a stage change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum TransitionCause {
    /// The learner pressed a button to move.
    LearnerChoice,
    /// The tutor (AI or rule) recommended the move.
    TutorRecommendation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageTransitionError {
    /// The tutor tried to skip stages without the learner asking.
    TutorCannotSkip {
        from: LearningStage,
        to: LearningStage,
    },
}

impl std::fmt::Display for StageTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageTransitionError::TutorCannotSkip { from, to } => write!(
                f,
                "the tutor cannot skip from \"{}\" to \"{}\"; only the learner can skip ahead",
                from.label(),
                to.label()
            ),
        }
    }
}

impl std::error::Error for StageTransitionError {}

impl LearningStage {
    pub const ALL: [LearningStage; 7] = [
        LearningStage::Introduction,
        LearningStage::ConcreteExample,
        LearningStage::ConceptExplanation,
        LearningStage::Formalisation,
        LearningStage::GuidedPractice,
        LearningStage::IndependentPractice,
        LearningStage::Reflection,
    ];

    /// Zero-based position in the journey.
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn first() -> Self {
        LearningStage::Introduction
    }

    pub fn next(self) -> Option<Self> {
        Self::ALL.get(self.index() + 1).copied()
    }

    pub fn previous(self) -> Option<Self> {
        self.index().checked_sub(1).map(|i| Self::ALL[i])
    }

    /// Check and perform a transition. Returns the new stage on success.
    pub fn transition_to(
        self,
        target: LearningStage,
        cause: TransitionCause,
    ) -> Result<LearningStage, StageTransitionError> {
        let from = self.index();
        let to = target.index();
        let is_skip = to > from + 1;
        if is_skip && cause == TransitionCause::TutorRecommendation {
            return Err(StageTransitionError::TutorCannotSkip { from: self, to: target });
        }
        Ok(target)
    }

    pub fn label(self) -> &'static str {
        match self {
            LearningStage::Introduction => "Introduce",
            LearningStage::ConcreteExample => "Make it concrete",
            LearningStage::ConceptExplanation => "Explain",
            LearningStage::Formalisation => "Formalise",
            LearningStage::GuidedPractice => "Practise together",
            LearningStage::IndependentPractice => "Practise on your own",
            LearningStage::Reflection => "Reflect",
        }
    }

    /// A short, plain-language description shown to the learner.
    pub fn description(self) -> &'static str {
        match self {
            LearningStage::Introduction => "What this topic is, and why it matters.",
            LearningStage::ConcreteExample => "A familiar example or worked case, before any theory.",
            LearningStage::ConceptExplanation => "The underlying idea, in plain language.",
            LearningStage::Formalisation => "The precise terms, notation and rules.",
            LearningStage::GuidedPractice => "Solve a problem together, one step at a time.",
            LearningStage::IndependentPractice => "Try a new problem without step-by-step help.",
            LearningStage::Reflection => "What feels clear, what is still uncertain, what to revisit.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use LearningStage::*;
    use TransitionCause::*;

    #[test]
    fn stages_are_in_design_order() {
        assert_eq!(LearningStage::first(), Introduction);
        assert_eq!(Introduction.next(), Some(ConcreteExample));
        assert_eq!(Reflection.next(), None);
        assert_eq!(Introduction.previous(), None);
        assert_eq!(Formalisation.previous(), Some(ConceptExplanation));
        for (i, s) in LearningStage::ALL.iter().enumerate() {
            assert_eq!(s.index(), i);
        }
    }

    #[test]
    fn anyone_can_move_back_any_distance() {
        for cause in [LearnerChoice, TutorRecommendation] {
            assert_eq!(Reflection.transition_to(Introduction, cause), Ok(Introduction));
            assert_eq!(GuidedPractice.transition_to(ConcreteExample, cause), Ok(ConcreteExample));
        }
    }

    #[test]
    fn anyone_can_repeat_or_step_forward_by_one() {
        for cause in [LearnerChoice, TutorRecommendation] {
            assert_eq!(Formalisation.transition_to(Formalisation, cause), Ok(Formalisation));
            assert_eq!(Formalisation.transition_to(GuidedPractice, cause), Ok(GuidedPractice));
        }
    }

    #[test]
    fn only_the_learner_can_skip_ahead() {
        assert_eq!(
            Introduction.transition_to(GuidedPractice, LearnerChoice),
            Ok(GuidedPractice)
        );
        assert_eq!(
            Introduction.transition_to(GuidedPractice, TutorRecommendation),
            Err(StageTransitionError::TutorCannotSkip { from: Introduction, to: GuidedPractice })
        );
    }

    #[test]
    fn every_stage_has_learner_facing_text() {
        for s in LearningStage::ALL {
            assert!(!s.label().is_empty());
            assert!(!s.description().is_empty());
        }
    }
}
