//! Teaching strategies the tutor can choose between (SDD §5.1).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum TeachingStrategy {
    ConcreteExample,
    Analogy,
    Visualisation,
    StepByStep,
    WorkedExample,
    Counterexample,
    Simplification,
    PrerequisiteReview,
    SocraticQuestioning,
}

impl TeachingStrategy {
    pub const ALL: [TeachingStrategy; 9] = [
        TeachingStrategy::ConcreteExample,
        TeachingStrategy::Analogy,
        TeachingStrategy::Visualisation,
        TeachingStrategy::StepByStep,
        TeachingStrategy::WorkedExample,
        TeachingStrategy::Counterexample,
        TeachingStrategy::Simplification,
        TeachingStrategy::PrerequisiteReview,
        TeachingStrategy::SocraticQuestioning,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TeachingStrategy::ConcreteExample => "Another example",
            TeachingStrategy::Analogy => "An analogy",
            TeachingStrategy::Visualisation => "Show it visually",
            TeachingStrategy::StepByStep => "Smaller steps",
            TeachingStrategy::WorkedExample => "A worked example",
            TeachingStrategy::Counterexample => "What it is not",
            TeachingStrategy::Simplification => "Explain it more simply",
            TeachingStrategy::PrerequisiteReview => "Review the basics first",
            TeachingStrategy::SocraticQuestioning => "Ask me a guiding question",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            TeachingStrategy::ConcreteExample => "Show a specific instance before generalising.",
            TeachingStrategy::Analogy => {
                "Relate the idea to something familiar, and say where the comparison stops working."
            }
            TeachingStrategy::Visualisation => "Use a diagram, a spatial picture or a sequence.",
            TeachingStrategy::StepByStep => "Break the explanation into smaller parts.",
            TeachingStrategy::WorkedExample => "Show a complete solution and why each step is taken.",
            TeachingStrategy::Counterexample => "Show a case that does not fit the definition or rule.",
            TeachingStrategy::Simplification => "Introduce fewer ideas at once.",
            TeachingStrategy::PrerequisiteReview => "Revisit an earlier concept this one depends on.",
            TeachingStrategy::SocraticQuestioning => "A focused question to help you reason it out yourself.",
        }
    }
}
