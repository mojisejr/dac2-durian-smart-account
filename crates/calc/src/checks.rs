use rust_decimal::Decimal;

use crate::{
    BusinessAnalysis, CheckKind, CheckResult, CheckStatus, CompletenessAnalysis, CostAnalysis,
    HealthAnalysis, HealthQuestion, Plan, Readiness, RevenueAnalysis,
};

pub fn calculate(
    plan: &Plan,
    revenue: &RevenueAnalysis,
    cost: &CostAnalysis,
    business: &BusinessAnalysis,
    health: &HealthAnalysis,
) -> CompletenessAnalysis {
    let checks = vec![
        equality_check(
            CheckKind::GradeSharesTotalOne,
            revenue.grade_share_total,
            Some(Decimal::ONE),
            Decimal::new(1, 4),
        ),
        positive_check(CheckKind::SellableYieldPositive, revenue.sellable_yield_kg),
        contribution_check(business.contribution_per_kg),
        positive_check(CheckKind::InvestmentPositive, cost.investment_base),
        linked_cost_check(cost),
        health_complete_check(plan, health),
    ];
    let overall = if checks.iter().all(|check| check.status == CheckStatus::Ok) {
        Readiness::Ready
    } else {
        Readiness::NeedsReview
    };

    CompletenessAnalysis { checks, overall }
}

fn equality_check(
    kind: CheckKind,
    actual: Option<Decimal>,
    expected: Option<Decimal>,
    tolerance: Decimal,
) -> CheckResult {
    let difference = actual
        .zip(expected)
        .map(|(actual, expected)| actual - expected);
    let status = match difference {
        Some(difference) if difference.abs() < tolerance => CheckStatus::Ok,
        Some(_) | None => CheckStatus::NeedsCheck,
    };
    CheckResult {
        kind,
        actual,
        expected,
        difference,
        status,
    }
}

fn positive_check(kind: CheckKind, actual: Option<Decimal>) -> CheckResult {
    let expected = Some(Decimal::ZERO);
    let status = match actual {
        Some(actual) if actual > Decimal::ZERO => CheckStatus::Ok,
        Some(_) | None => CheckStatus::NeedsCheck,
    };
    CheckResult {
        kind,
        actual,
        expected,
        difference: actual.map(|actual| actual - Decimal::ZERO),
        status,
    }
}

fn contribution_check(contribution: Option<Decimal>) -> CheckResult {
    let status = match contribution {
        Some(value) if value > Decimal::ZERO => CheckStatus::Ok,
        Some(_) => CheckStatus::Warning,
        None => CheckStatus::NeedsCheck,
    };
    CheckResult {
        kind: CheckKind::PriceAboveVariableCost,
        actual: contribution,
        expected: Some(Decimal::ZERO),
        difference: contribution,
        status,
    }
}

fn linked_cost_check(cost: &CostAnalysis) -> CheckResult {
    equality_check(
        CheckKind::TotalCostLinked,
        cost.total_cost,
        cost.variable_cost
            .zip(cost.fixed_cost)
            .map(|(variable, fixed)| variable + fixed),
        Decimal::new(1, 2),
    )
}

fn health_complete_check(plan: &Plan, health: &HealthAnalysis) -> CheckResult {
    let answered = HealthQuestion::ALL
        .into_iter()
        .filter(|question| {
            let matching: Vec<_> = plan
                .health_answers
                .iter()
                .filter(|answer| answer.question == *question)
                .collect();
            matching.len() == 1
                && matching[0]
                    .score
                    .is_some_and(|score| (1..=5).contains(&score))
        })
        .count();
    let actual = Some(Decimal::from(answered));
    let expected = Some(Decimal::from(12));
    let difference = actual
        .zip(expected)
        .map(|(actual, expected)| actual - expected);
    let status = if answered == 12 && health.overall_score.is_some() {
        CheckStatus::Ok
    } else {
        CheckStatus::NeedsCheck
    };
    CheckResult {
        kind: CheckKind::HealthAnswersComplete,
        actual,
        expected,
        difference,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{analyze, cost, health, revenue, workbook_sample};

    fn status(plan: &Plan, kind: CheckKind) -> CheckStatus {
        analyze(plan)
            .checks
            .checks
            .into_iter()
            .find(|check| check.kind == kind)
            .expect("all six checks")
            .status
    }

    #[test]
    fn workbook_is_ready_with_all_six_checks_ok() {
        let result = analyze(&workbook_sample()).checks;
        assert_eq!(result.checks.len(), 6);
        assert!(
            result
                .checks
                .iter()
                .all(|check| check.status == CheckStatus::Ok)
        );
        assert_eq!(result.overall, Readiness::Ready);
    }

    #[test]
    fn each_owner_input_check_can_fail_independently() {
        let mut plan = workbook_sample();
        plan.production.grades[0].share = Some(Decimal::new(49, 2));
        assert_eq!(
            status(&plan, CheckKind::GradeSharesTotalOne),
            CheckStatus::NeedsCheck
        );

        plan = workbook_sample();
        plan.production.producing_trees = Some(Decimal::ZERO);
        assert_eq!(
            status(&plan, CheckKind::SellableYieldPositive),
            CheckStatus::NeedsCheck
        );

        plan = workbook_sample();
        for grade in &mut plan.production.grades {
            grade.price_per_kg = Some(Decimal::ZERO);
        }
        assert_eq!(
            status(&plan, CheckKind::PriceAboveVariableCost),
            CheckStatus::Warning
        );

        plan = workbook_sample();
        for line in &mut plan.fixed_costs {
            line.investment_base = Some(Decimal::ZERO);
        }
        assert_eq!(
            status(&plan, CheckKind::InvestmentPositive),
            CheckStatus::NeedsCheck
        );

        plan = workbook_sample();
        plan.health_answers.pop();
        assert_eq!(
            status(&plan, CheckKind::HealthAnswersComplete),
            CheckStatus::NeedsCheck
        );
    }

    #[test]
    fn broken_cost_link_is_detected_by_its_own_rule() {
        let plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let mut cost = cost::calculate(&plan, &revenue);
        cost.total_cost = cost.total_cost.map(|value| value + Decimal::ONE);
        assert_eq!(linked_cost_check(&cost).status, CheckStatus::NeedsCheck);

        let health = health::calculate(&plan);
        let business = crate::breakeven::calculate(&revenue, &cost);
        let result = calculate(&plan, &revenue, &cost, &business, &health);
        assert_eq!(result.overall, Readiness::NeedsReview);
    }
}
