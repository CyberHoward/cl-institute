use crate::constraints::{
    AuthorityLevelCheck, EdgeNodeReference, GraphConnectivity, NoCyclicDependency,
    RequiredPolicyScope, UniqueNames,
};
use crate::traits::Constraint;
use crate::types::{ValidationContext, ValidationResult};

/// The constraint engine collects constraints and runs them all against
/// a `ValidationContext`, producing an aggregated `ValidationResult`.
pub struct ConstraintEngine {
    constraints: Vec<Box<dyn Constraint>>,
}

impl ConstraintEngine {
    /// Create an empty engine with no constraints registered.
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Create an engine pre-loaded with all built-in constraints.
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();
        engine.add_constraint(GraphConnectivity);
        engine.add_constraint(AuthorityLevelCheck);
        engine.add_constraint(EdgeNodeReference);
        engine.add_constraint(RequiredPolicyScope);
        engine.add_constraint(NoCyclicDependency);
        engine.add_constraint(UniqueNames);
        engine
    }

    /// Register an additional constraint.
    pub fn add_constraint(
        &mut self,
        c: impl Constraint + 'static,
    ) {
        self.constraints.push(Box::new(c));
    }

    /// Run every registered constraint against the context and
    /// return the aggregated result.
    pub fn validate(&self, ctx: &ValidationContext) -> ValidationResult {
        let mut all_violations = Vec::new();

        for constraint in &self.constraints {
            if let Err(violations) = constraint.validate(ctx) {
                all_violations.extend(violations);
            }
        }

        ValidationResult::from_violations(all_violations)
    }
}

impl Default for ConstraintEngine {
    fn default() -> Self {
        Self::new()
    }
}
