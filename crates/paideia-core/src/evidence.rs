//! Dimensions of learning evidence (design doc §5, "Learner model").
//!
//! Progress is never reduced to one score. Each dimension is tracked separately,
//! because success in one does not guarantee success in another.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EvidenceDimension {
    Recognition,
    Explanation,
    Recall,
    Application,
    Transfer,
}

impl EvidenceDimension {
    pub const ALL: [EvidenceDimension; 5] = [
        EvidenceDimension::Recognition,
        EvidenceDimension::Explanation,
        EvidenceDimension::Recall,
        EvidenceDimension::Application,
        EvidenceDimension::Transfer,
    ];

    /// The question each dimension answers, phrased for the learner.
    pub fn question(self) -> &'static str {
        match self {
            EvidenceDimension::Recognition => "Can you spot it, or pick out a correct example?",
            EvidenceDimension::Explanation => "Can you explain it in your own words?",
            EvidenceDimension::Recall => "Can you remember it without seeing the answer?",
            EvidenceDimension::Application => "Can you use it on a familiar problem?",
            EvidenceDimension::Transfer => "Can you use it somewhere new?",
        }
    }
}
