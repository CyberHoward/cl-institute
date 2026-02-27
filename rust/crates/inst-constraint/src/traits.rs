use crate::types::ConstraintViolation;

/// Core abstraction for institutional invariant validation.
///
/// Each constraint is a typed predicate that examines some context
/// and produces zero or more violations. Constraints are composable:
/// the `ConstraintEngine` collects them and runs them all against
/// the same `ValidationContext`.
pub trait Constraint {
    /// The data this constraint needs to evaluate.
    type Context;

    /// Validate the context, returning `Ok(())` if no violations
    /// are found, or `Err(violations)` listing every issue detected.
    fn validate(&self, ctx: &Self::Context) -> Result<(), Vec<ConstraintViolation>>;

    /// A short, unique name for this constraint (e.g., `"graph_connectivity"`).
    fn name(&self) -> &str;

    /// A human-readable explanation of what this constraint checks.
    fn description(&self) -> &str;
}
