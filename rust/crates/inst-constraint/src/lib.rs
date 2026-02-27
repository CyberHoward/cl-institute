//! # inst-constraint
//!
//! The constraint engine for the Intelligent Institution Initiative.
//!
//! This crate validates institutional invariants (Layer 1 hard constraints)
//! against the organizational model defined in `inst-model`. Constraints are
//! evaluated both at definition time (when a workflow is created or modified)
//! and at runtime (when decisions are executed).
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use inst_constraint::{ConstraintEngine, ValidationContext};
//!
//! let engine = ConstraintEngine::with_defaults();
//! // ... build a ValidationContext from your model data ...
//! // let result = engine.validate(&ctx);
//! // assert!(result.is_valid);
//! ```

pub mod constraints;
pub mod engine;
pub mod traits;
pub mod types;

// Re-export the public API at crate root for convenience.
pub use constraints::{
    AuthorityLevelCheck, EdgeNodeReference, GraphConnectivity, NoCyclicDependency,
    RequiredPolicyScope, UniqueNames,
};
pub use engine::ConstraintEngine;
pub use traits::Constraint;
pub use types::{
    ConstraintViolation, Severity, ValidationContext, ValidationResult, ViolationLocation,
    WorkflowData,
};
