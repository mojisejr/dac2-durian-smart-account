use rust_decimal::Decimal;

use crate::{CashKind, CostAnalysis, Plan, RevenueAnalysis, VariableCostKind, VariableCostResult};

pub fn calculate(plan: &Plan, revenue: &RevenueAnalysis) -> CostAnalysis {
    let mut variable_lines: Vec<_> = plan
        .variable_costs
        .iter()
        .map(|line| {
            let quantity = line.effective_quantity(revenue.sellable_yield_kg);
            VariableCostResult {
                kind: line.kind,
                quantity,
                total: quantity
                    .zip(line.unit_price)
                    .map(|(quantity, price)| quantity * price),
                yield_per_unit: revenue
                    .sellable_yield_kg
                    .zip(quantity)
                    .and_then(|(yield_kg, quantity)| nonzero_ratio(yield_kg, quantity)),
                share_of_variable_cost: None,
            }
        })
        .collect();

    let variable_cost = complete_sum(variable_lines.iter().map(|line| line.total));
    for line in &mut variable_lines {
        line.share_of_variable_cost = line
            .total
            .zip(variable_cost)
            .and_then(|(total, variable_cost)| nonzero_ratio(total, variable_cost));
    }

    let variable_cost_per_kg = variable_cost
        .zip(revenue.sellable_yield_kg)
        .and_then(|(cost, yield_kg)| nonzero_ratio(cost, yield_kg));
    let fixed_cost = complete_sum(plan.fixed_costs.iter().map(|line| line.amount_per_year));
    let cash_fixed_cost = fixed_cost.map(|_| {
        plan.fixed_costs
            .iter()
            .filter(|line| line.cash_kind == CashKind::Cash)
            .filter_map(|line| line.amount_per_year)
            .sum()
    });
    let investment_base = complete_sum(plan.fixed_costs.iter().map(|line| line.investment_base));
    let total_cost = variable_cost
        .zip(fixed_cost)
        .map(|(variable, fixed)| variable + fixed);
    let cost_per_kg = total_cost
        .zip(revenue.sellable_yield_kg)
        .and_then(|(cost, yield_kg)| nonzero_ratio(cost, yield_kg));
    let cost_per_rai = total_cost
        .zip(plan.production.area_rai)
        .and_then(|(cost, rai)| nonzero_ratio(cost, rai));

    CostAnalysis {
        variable_lines,
        area_rai: plan.production.area_rai,
        variable_cost,
        variable_cost_per_kg,
        fixed_cost,
        cash_fixed_cost,
        investment_base,
        total_cost,
        cost_per_kg,
        cost_per_rai,
    }
}

fn complete_sum(values: impl IntoIterator<Item = Option<Decimal>>) -> Option<Decimal> {
    let values: Vec<_> = values.into_iter().collect();
    if values.is_empty() {
        return None;
    }
    values.into_iter().try_fold(Decimal::ZERO, |total, value| {
        value.map(|value| total + value)
    })
}

fn nonzero_ratio(numerator: Decimal, denominator: Decimal) -> Option<Decimal> {
    (!denominator.is_zero()).then(|| numerator / denominator)
}

pub(crate) fn quantity_for_kind(
    plan: &Plan,
    revenue: &RevenueAnalysis,
    kind: VariableCostKind,
) -> Option<Decimal> {
    let matching: Vec<_> = plan
        .variable_costs
        .iter()
        .filter(|line| line.kind == kind)
        .map(|line| line.effective_quantity(revenue.sellable_yield_kg))
        .collect();
    complete_sum(matching)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FixedCostLine, VariableCostLine, revenue, workbook_sample};

    #[test]
    fn workbook_cost_rollup_and_cash_split_match() {
        let plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let result = calculate(&plan, &revenue);

        assert_eq!(result.variable_cost, Some(Decimal::from(556_175)));
        assert_eq!(result.fixed_cost, Some(Decimal::from(255_100)));
        assert_eq!(result.cash_fixed_cost, Some(Decimal::from(192_000)));
        assert_eq!(result.investment_base, Some(Decimal::from(700_000)));
        assert_eq!(result.total_cost, Some(Decimal::from(811_275)));
        assert_eq!(result.cost_per_rai, Some(Decimal::new(811_275, 1)));
    }

    #[test]
    fn linked_cost_quantities_follow_yield_and_allow_an_explicit_override() {
        let mut plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let harvest = plan
            .variable_costs
            .iter()
            .position(|line| line.kind == VariableCostKind::HarvestLabor)
            .expect("sample harvest line");

        let derived = calculate(&plan, &revenue);
        assert_eq!(
            derived.variable_lines[harvest].quantity,
            Some(Decimal::from(19_950))
        );

        plan.variable_costs[harvest].quantity = Some(Decimal::from(10_000));
        let overridden = calculate(&plan, &revenue);
        assert_eq!(
            overridden.variable_lines[harvest].quantity,
            Some(Decimal::from(10_000))
        );
    }

    #[test]
    fn every_cost_kind_rolls_up_and_other_has_no_special_exclusion() {
        let kinds = [
            VariableCostKind::Fertilizer,
            VariableCostKind::CropProtection,
            VariableCostKind::Water,
            VariableCostKind::OrchardLabor,
            VariableCostKind::Electricity,
            VariableCostKind::Fuel,
            VariableCostKind::HarvestLabor,
            VariableCostKind::Transport,
            VariableCostKind::Packing,
            VariableCostKind::Maintenance,
            VariableCostKind::Other,
        ];
        let mut plan = workbook_sample();
        plan.variable_costs = kinds
            .into_iter()
            .map(|kind| VariableCostLine {
                name: format!("{kind:?}"),
                kind,
                quantity: Some(Decimal::ONE),
                unit: "หน่วย".into(),
                unit_price: Some(Decimal::ONE),
            })
            .collect();
        let revenue = revenue::calculate(&plan);
        assert_eq!(
            calculate(&plan, &revenue).variable_cost,
            Some(Decimal::from(11))
        );
    }

    #[test]
    fn missing_lines_and_zero_denominators_are_unavailable() {
        let mut plan = workbook_sample();
        plan.variable_costs.clear();
        plan.fixed_costs.clear();
        plan.production.producing_trees = Some(Decimal::ZERO);
        let revenue = revenue::calculate(&plan);
        let result = calculate(&plan, &revenue);

        assert_eq!(result.variable_cost, None);
        assert_eq!(result.fixed_cost, None);
        assert_eq!(result.cost_per_kg, None);
    }

    #[test]
    fn zero_or_missing_area_has_no_per_rai_result() {
        let mut plan = workbook_sample();
        plan.production.area_rai = Some(Decimal::ZERO);
        let revenue = revenue::calculate(&plan);
        assert_eq!(calculate(&plan, &revenue).cost_per_rai, None);

        plan.production.area_rai = None;
        let revenue = revenue::calculate(&plan);
        assert_eq!(calculate(&plan, &revenue).cost_per_rai, None);
    }

    #[test]
    fn a_missing_value_keeps_the_whole_rollup_incomplete() {
        let mut plan = workbook_sample();
        plan.fixed_costs.push(FixedCostLine {
            name: "ยังไม่กรอก".into(),
            cash_kind: CashKind::Cash,
            amount_per_year: None,
            investment_base: Some(Decimal::ZERO),
        });
        let revenue = revenue::calculate(&plan);
        assert_eq!(calculate(&plan, &revenue).fixed_cost, None);
    }

    #[test]
    fn complete_fixed_costs_with_no_cash_rows_have_zero_cash_cost() {
        let mut plan = workbook_sample();
        for line in &mut plan.fixed_costs {
            line.cash_kind = CashKind::NonCash;
        }
        let revenue = revenue::calculate(&plan);
        assert_eq!(
            calculate(&plan, &revenue).cash_fixed_cost,
            Some(Decimal::ZERO)
        );
    }
}
