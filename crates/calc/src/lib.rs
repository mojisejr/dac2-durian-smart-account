#![forbid(unsafe_code)]

//! Pure calculation boundary for DAC2.
//!
//! The crate has no I/O, async runtime, clock, environment, or feature gate. A
//! [`Plan`] is transformed deterministically into an [`Analysis`] by
//! [`analyze`], and the same code builds for native Rust and WebAssembly.

pub mod actual;
pub mod analysis;
pub mod breakeven;
pub mod checks;
pub mod comparison;
pub mod cost;
pub mod efficiency;
pub mod health;
pub mod plan;
pub mod quick;
pub mod revenue;
pub mod sample;
pub mod scenario;
pub mod targets;
pub mod tax;
pub mod trend;
pub mod validation;

pub use actual::*;
pub use analysis::*;
pub use comparison::*;
pub use plan::*;
pub use quick::*;
pub use sample::workbook_sample;
pub use targets::*;
pub use trend::*;
pub use validation::*;

/// Calculate every derived workbook surface from one input aggregate.
pub fn analyze(plan: &Plan) -> Analysis {
    let revenue = revenue::calculate(plan);
    let cost = cost::calculate(plan, &revenue);
    let business = breakeven::calculate(&revenue, &cost);
    let health = health::calculate(plan);
    let efficiency = efficiency::calculate(plan, &revenue, &cost);
    let tax = tax::calculate(&revenue, &cost);
    let scenario = scenario::calculate(&revenue, &cost);
    let checks = checks::calculate(plan, &revenue, &cost, &business, &health);

    Analysis {
        input_issues: plan.input_issues(),
        revenue,
        cost,
        business,
        efficiency,
        tax,
        scenario,
        checks,
        health,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_plan_is_explicitly_incomplete_and_never_panics() {
        let result = analyze(&Plan::default());

        assert!(result.revenue.sellable_yield_kg.is_none());
        assert!(result.business.net_profit.is_none());
        assert_eq!(result.checks.overall, Readiness::NeedsReview);
    }
}
