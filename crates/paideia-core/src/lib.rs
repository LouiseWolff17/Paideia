//! Paideia domain layer.
//!
//! This crate holds the core learning concepts and the rules that govern them.
//! It must stay independent of the UI framework, the database and the AI provider
//! (design doc §6, "Architectural boundaries").

pub mod api;
pub mod evidence;
pub mod stage;
pub mod strategy;

pub use evidence::EvidenceDimension;
pub use stage::{LearningStage, StageTransitionError, TransitionCause};
pub use strategy::TeachingStrategy;

/// The six main areas of the application (design doc §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Area {
    Learn,
    Remember,
    Explore,
    MyTasks,
    Progress,
    Preferences,
}

impl Area {
    pub const ALL: [Area; 6] = [
        Area::Learn,
        Area::Remember,
        Area::Explore,
        Area::MyTasks,
        Area::Progress,
        Area::Preferences,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Area::Learn => "Learn",
            Area::Remember => "Remember",
            Area::Explore => "Explore",
            Area::MyTasks => "My Tasks",
            Area::Progress => "Progress",
            Area::Preferences => "Preferences",
        }
    }

    pub fn purpose(self) -> &'static str {
        match self {
            Area::Learn => "Continue a learning session, study a concept or start something new.",
            Area::Remember => "Review what you have learned with recall practice, at a gentle pace.",
            Area::Explore => "Browse your subjects, concepts and learning materials.",
            Area::MyTasks => "Turn a big study task into small next steps, and pick up where you left off.",
            Area::Progress => "See what you can do so far, and what is ready for review.",
            Area::Preferences => "Choose how things look, how explanations are given, and how AI is used.",
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Area::Learn => "/learn",
            Area::Remember => "/remember",
            Area::Explore => "/explore",
            Area::MyTasks => "/tasks",
            Area::Progress => "/progress",
            Area::Preferences => "/preferences",
        }
    }
}
