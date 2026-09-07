use rust_decimal::Decimal;

use crate::{CostAnalysis, RevenueAnalysis, ScenarioAnalysis};

const CHANGES: [Decimal; 5] = [
    Decimal::from_parts(5, 0, 0, true, 1),
    Decimal::from_parts(1, 0, 0, true, 1),
    Decimal::ZERO,
    Decimal::from_parts(1, 0, 0, false, 1),
    Decimal::from_parts(2, 0, 0, false, 1),
];

// The workbook's first price header says -20%, but its controlling cell B6 is
// -50% and all cached values use -50%. Parity follows the value, not the label.

pub fn calculate(revenue: &RevenueAnalysis, cost: &CostAnalysis) -> ScenarioAnalysis {
    let profits = std::array::from_fn(|yield_index| {
        std::array::from_fn(|price_index| {
            profit_at(revenue, cost, CHANGES[yield_index], CHANGES[price_index])
        })
    });

    ScenarioAnalysis {
        yield_changes: CHANGES,
        price_changes: CHANGES,
        profits,
    }
}

pub fn profit_at(
    revenue: &RevenueAnalysis,
    cost: &CostAnalysis,
    yield_change: Decimal,
    price_change: Decimal,
) -> Option<Decimal> {
    let adjusted_yield = revenue
        .sellable_yield_kg?
        .checked_mul(Decimal::ONE + yield_change)?;
    let adjusted_price = revenue
        .weighted_price_per_kg?
        .checked_mul(Decimal::ONE + price_change)?;
    let adjusted_revenue = adjusted_yield.checked_mul(adjusted_price)?;
    let adjusted_variable_cost = cost
        .variable_cost?
        .checked_mul(Decimal::ONE + yield_change)?;
    Some(adjusted_revenue - cost.fixed_cost? - adjusted_variable_cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cost, revenue, workbook_sample};

    #[test]
    fn workbook_base_and_extreme_scenarios_match() {
        let plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let cost = cost::calculate(&plan, &revenue);
        let result = calculate(&revenue, &cost);

        assert_eq!(result.profits[2][2], Some(Decimal::from(834_600)));
        assert_eq!(result.profits[0][0], Some(Decimal::new(-12_171_875, 2)));
        assert_eq!(result.profits[4][4], Some(Decimal::from(1_447_550)));
    }

    #[test]
    fn missing_inputs_leave_all_twenty_five_cells_unavailable() {
        let mut plan = workbook_sample();
        plan.production.grades[0].price_per_kg = None;
        let revenue = revenue::calculate(&plan);
        let cost = cost::calculate(&plan, &revenue);
        let result = calculate(&revenue, &cost);

        assert!(result.profits.iter().flatten().all(Option::is_none));
    }

    #[test]
    fn full_precision_inputs_are_used_without_presentation_rounding() {
        let revenue = RevenueAnalysis {
            gross_yield_kg: None,
            sellable_yield_kg: Some(Decimal::ONE),
            market_gap_kg: None,
            grade_share_total: None,
            weighted_price_per_kg: Some(Decimal::new(1005, 3)),
            revenue: None,
            market_fulfillment: None,
        };
        let cost = CostAnalysis {
            variable_lines: Vec::new(),
            area_rai: None,
            variable_cost: Some(Decimal::ZERO),
            variable_cost_per_kg: None,
            fixed_cost: Some(Decimal::ZERO),
            cash_fixed_cost: Some(Decimal::ZERO),
            investment_base: Some(Decimal::ZERO),
            total_cost: Some(Decimal::ZERO),
            cost_per_kg: None,
            cost_per_rai: None,
        };
        assert_eq!(
            calculate(&revenue, &cost).profits[2][2],
            Some(Decimal::new(1005, 3))
        );
    }
}
