//! Budget enforcement primitives.
//!
//! Validates whether a proposed spend fits within the session's remaining
//! budget and per-request cap.

use pennykite_types::{SessionPolicy, Verdict};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BudgetError {
    #[error("per-request cap exceeded: {estimated} > {cap}")]
    PerRequestCapExceeded { estimated: f64, cap: f64 },
    #[error("session budget exceeded: {spent} + {estimated} > {budget}")]
    SessionBudgetExceeded {
        spent: f64,
        estimated: f64,
        budget: f64,
    },
}

/// Check whether a proposed spend fits within the session policy.
///
/// Returns `Ok(())` if the spend is allowed, or a `BudgetError` describing
/// which limit was exceeded.
pub fn check_budget(
    policy: &SessionPolicy,
    spent_so_far: f64,
    estimated_cost: f64,
) -> Result<(), BudgetError> {
    if estimated_cost > policy.per_request_cap_usd {
        return Err(BudgetError::PerRequestCapExceeded {
            estimated: estimated_cost,
            cap: policy.per_request_cap_usd,
        });
    }
    if spent_so_far + estimated_cost > policy.budget_usd {
        return Err(BudgetError::SessionBudgetExceeded {
            spent: spent_so_far,
            estimated: estimated_cost,
            budget: policy.budget_usd,
        });
    }
    Ok(())
}

/// Map a budget check result to a verdict.
pub fn budget_verdict(
    policy: &SessionPolicy,
    spent_so_far: f64,
    estimated_cost: f64,
) -> Verdict {
    match check_budget(policy, spent_so_far, estimated_cost) {
        Ok(()) => Verdict::Approve,
        Err(_) => Verdict::DenyBudget,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy() -> SessionPolicy {
        SessionPolicy {
            budget_usd: 5.0,
            per_request_cap_usd: 0.5,
            max_duration_minutes: 30,
        }
    }

    #[test]
    fn approve_within_limits() {
        let p = test_policy();
        assert!(check_budget(&p, 1.0, 0.05).is_ok());
    }

    #[test]
    fn deny_per_request_cap() {
        let p = test_policy();
        let err = check_budget(&p, 0.0, 1.0).unwrap_err();
        assert!(matches!(err, BudgetError::PerRequestCapExceeded { .. }));
    }

    #[test]
    fn deny_session_budget() {
        let p = test_policy();
        let err = check_budget(&p, 4.99, 0.05).unwrap_err();
        assert!(matches!(err, BudgetError::SessionBudgetExceeded { .. }));
    }

    #[test]
    fn budget_verdict_approve() {
        let p = test_policy();
        assert_eq!(budget_verdict(&p, 1.0, 0.05), Verdict::Approve);
    }

    #[test]
    fn budget_verdict_deny() {
        let p = test_policy();
        assert_eq!(budget_verdict(&p, 5.0, 0.01), Verdict::DenyBudget);
    }
}
