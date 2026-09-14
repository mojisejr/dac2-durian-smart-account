use rust_decimal::Decimal;

use crate::{
    AssetAllocation, CashKind, CostAnalysis, CostSectionState, Plan, RevenueAnalysis,
    VariableCostKind, VariableCostResult,
};

pub fn calculate(plan: &Plan, revenue: &RevenueAnalysis) -> CostAnalysis {
    calculate_with_assets(plan, revenue, &[], None)
}

pub fn calculate_with_assets(
    plan: &Plan,
    revenue: &RevenueAnalysis,
    assets: &[AssetAllocation],
    starting_capital: Option<Decimal>,
) -> CostAnalysis {
    let mut variable_lines: Vec<_> = plan
        .variable_costs
        .iter()
        .map(|line| {
            let quantity = line.effective_quantity(revenue.sellable_yield_kg);
            VariableCostResult {
                kind: line.kind,
                quantity,
                total: line.total(revenue.sellable_yield_kg),
                yield_per_unit: revenue
                    .sellable_yield_kg
                    .zip(quantity)
                    .and_then(|(yield_kg, quantity)| nonzero_ratio(yield_kg, quantity)),
                share_of_variable_cost: None,
            }
        })
        .collect();

    // A confirmed-none section is a known zero; an unknown one stays
    // unavailable and withholds only the results that depend on it.
    let variable_cost = match plan.effective_variable_cost_state() {
        CostSectionState::EnteredItems => {
            complete_sum(variable_lines.iter().map(|line| line.total))
        }
        CostSectionState::ConfirmedNone => Some(Decimal::ZERO),
        CostSectionState::Unknown => None,
    };
    for line in &mut variable_lines {
        line.share_of_variable_cost = line
            .total
            .zip(variable_cost)
            .and_then(|(total, variable_cost)| nonzero_ratio(total, variable_cost));
    }

    let variable_cost_per_kg = variable_cost
        .zip(revenue.sellable_yield_kg)
        .and_then(|(cost, yield_kg)| nonzero_ratio(cost, yield_kg));
    let fixed_state = plan.effective_fixed_cost_state();
    let manual_fixed_cost = match fixed_state {
        CostSectionState::EnteredItems => {
            complete_sum(plan.fixed_costs.iter().map(|line| line.amount_per_year))
        }
        CostSectionState::ConfirmedNone => Some(Decimal::ZERO),
        CostSectionState::Unknown => None,
    };
    let asset_depreciation =
        present_sum(assets.iter().map(|asset| Some(asset.annual_depreciation)));
    // Depreciation from selected assets is added to what the owner said about
    // the manual section. An unknown manual section keeps the total unknown
    // even when assets exist: depreciation alone is not the whole fixed cost.
    let fixed_cost =
        manual_fixed_cost.map(|manual| manual + asset_depreciation.unwrap_or(Decimal::ZERO));
    let cash_fixed_cost = manual_fixed_cost.map(|_| {
        plan.fixed_costs
            .iter()
            .filter(|line| line.cash_kind == CashKind::Cash)
            .filter_map(|line| line.amount_per_year)
            .sum()
    });
    let manual_investment_base =
        present_sum(plan.fixed_costs.iter().map(|line| line.investment_base));
    let asset_investment_base =
        present_sum(assets.iter().map(|asset| Some(asset.investment_value)));
    let investment_base = present_sum([
        manual_investment_base,
        asset_investment_base,
        starting_capital,
    ]);
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
        manual_fixed_cost,
        asset_depreciation,
        fixed_cost,
        cash_fixed_cost,
        manual_investment_base,
        asset_investment_base,
        starting_capital,
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

fn present_sum(values: impl IntoIterator<Item = Option<Decimal>>) -> Option<Decimal> {
    let mut found = false;
    let total = values
        .into_iter()
        .flatten()
        .fold(Decimal::ZERO, |total, value| {
            found = true;
            total + value
        });
    found.then_some(total)
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
    use crate::{FixedCostLine, UnclassifiedExpense, VariableCostLine, revenue, workbook_sample};

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
                total_amount: None,
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

    #[test]
    fn blank_investment_on_a_recurring_cost_does_not_hide_other_investments() {
        let mut blank = workbook_sample();
        blank.fixed_costs.push(FixedCostLine {
            name: "ค่าแรงประจำเพิ่ม".into(),
            cash_kind: CashKind::Cash,
            amount_per_year: Some(Decimal::from(60_000)),
            investment_base: None,
        });
        let mut explicit_zero = blank.clone();
        explicit_zero
            .fixed_costs
            .last_mut()
            .expect("the recurring cost exists")
            .investment_base = Some(Decimal::ZERO);

        let blank_result = crate::analyze(&blank);
        let zero_result = crate::analyze(&explicit_zero);

        assert_eq!(
            blank_result.cost.investment_base,
            Some(Decimal::from(700_000))
        );
        assert_eq!(blank_result.business.roi, zero_result.business.roi);
        assert_eq!(
            blank_result.business.payback_years,
            zero_result.business.payback_years
        );
    }

    #[test]
    fn all_blank_investments_keep_roi_and_payback_unavailable() {
        let mut plan = workbook_sample();
        for line in &mut plan.fixed_costs {
            line.investment_base = None;
        }

        let result = crate::analyze(&plan);

        assert_eq!(result.cost.investment_base, None);
        assert_eq!(result.business.roi, None);
        assert_eq!(result.business.payback_years, None);
    }

    #[test]
    fn asset_depreciation_and_starting_capital_keep_distinct_roles() {
        let plan = workbook_sample();
        let asset = crate::allocate(
            &crate::AssetFacts {
                name: "ระบบน้ำ".into(),
                kind: crate::AssetKind::Equipment,
                original_cost: Decimal::from(100_000),
                start_year: 2568,
                useful_life_years: Some(5),
                residual_value: None,
                retired_year: None,
            },
            2569,
        )
        .unwrap()
        .unwrap();

        let without = crate::analyze(&plan);
        let with = crate::analyze_with_assets(&plan, &[asset], Some(Decimal::from(50_000)));

        assert_eq!(with.cost.manual_fixed_cost, without.cost.fixed_cost);
        assert_eq!(with.cost.asset_depreciation, Some(Decimal::from(20_000)));
        assert_eq!(
            with.cost.total_cost,
            without
                .cost
                .total_cost
                .map(|value| value + Decimal::from(20_000))
        );
        assert_eq!(with.cost.cash_fixed_cost, without.cost.cash_fixed_cost);
        assert_eq!(
            with.business.operating_cash_flow,
            without.business.operating_cash_flow
        );
        assert_eq!(
            with.cost.manual_investment_base,
            without.cost.investment_base
        );
        assert_eq!(
            with.cost.asset_investment_base,
            Some(Decimal::from(100_000))
        );
        assert_eq!(with.cost.starting_capital, Some(Decimal::from(50_000)));
        assert_eq!(with.cost.investment_base, Some(Decimal::from(850_000)));
    }

    #[test]
    fn asset_only_fixed_cost_has_a_known_zero_cash_component() {
        let mut plan = workbook_sample();
        plan.fixed_costs.clear();
        plan.fixed_cost_state = CostSectionState::ConfirmedNone;
        let asset = crate::allocate(
            &crate::AssetFacts {
                name: "ระบบน้ำ".into(),
                kind: crate::AssetKind::Equipment,
                original_cost: Decimal::from(100_000),
                start_year: 2568,
                useful_life_years: Some(5),
                residual_value: None,
                retired_year: None,
            },
            2569,
        )
        .unwrap()
        .unwrap();

        let result = crate::analyze_with_assets(&plan, &[asset], None);

        assert_eq!(result.cost.fixed_cost, Some(Decimal::from(20_000)));
        assert_eq!(result.cost.cash_fixed_cost, Some(Decimal::ZERO));
        assert_eq!(
            result.business.operating_cash_flow,
            result
                .revenue
                .revenue
                .zip(result.cost.variable_cost)
                .map(|(revenue, variable)| revenue - variable)
        );
    }

    fn asset_allocation() -> AssetAllocation {
        crate::allocate(
            &crate::AssetFacts {
                name: "ระบบน้ำ".into(),
                kind: crate::AssetKind::Equipment,
                original_cost: Decimal::from(100_000),
                start_year: 2568,
                useful_life_years: Some(5),
                residual_value: None,
                retired_year: None,
            },
            2569,
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn confirmed_none_is_a_known_zero_and_unknown_withholds_only_dependents() {
        let mut plan = workbook_sample();
        plan.variable_costs.clear();
        plan.fixed_costs.clear();
        let revenue = revenue::calculate(&plan);

        plan.variable_cost_state = CostSectionState::ConfirmedNone;
        plan.fixed_cost_state = CostSectionState::ConfirmedNone;
        let none = calculate(&plan, &revenue);
        assert_eq!(none.variable_cost, Some(Decimal::ZERO));
        assert_eq!(none.fixed_cost, Some(Decimal::ZERO));
        assert_eq!(none.cash_fixed_cost, Some(Decimal::ZERO));
        assert_eq!(none.total_cost, Some(Decimal::ZERO));
        assert_eq!(none.cost_per_kg, Some(Decimal::ZERO));
        assert_eq!(
            none.investment_base, None,
            "confirmed none invents no investment"
        );

        plan.fixed_cost_state = CostSectionState::Unknown;
        let half = calculate(&plan, &revenue);
        assert_eq!(half.variable_cost, Some(Decimal::ZERO));
        assert_eq!(half.fixed_cost, None);
        assert_eq!(half.total_cost, None);
        assert_eq!(half.cost_per_kg, None);
        assert_eq!(
            revenue.revenue,
            Some(Decimal::from(1_645_875)),
            "revenue does not depend on the cost sections"
        );
    }

    #[test]
    fn an_unknown_fixed_section_stays_unknown_even_with_asset_depreciation() {
        let mut plan = workbook_sample();
        plan.fixed_costs.clear();
        let asset = asset_allocation();

        let unknown = crate::analyze_with_assets(&plan, &[asset.clone()], None);
        assert_eq!(unknown.cost.asset_depreciation, Some(Decimal::from(20_000)));
        assert_eq!(unknown.cost.fixed_cost, None);
        assert_eq!(unknown.cost.cash_fixed_cost, None);

        plan.fixed_cost_state = CostSectionState::ConfirmedNone;
        let confirmed = crate::analyze_with_assets(&plan, &[asset], None);
        assert_eq!(confirmed.cost.fixed_cost, Some(Decimal::from(20_000)));
        assert_eq!(confirmed.cost.cash_fixed_cost, Some(Decimal::ZERO));
    }

    #[test]
    fn rows_always_win_over_a_stored_section_state() {
        let mut plan = workbook_sample();
        plan.variable_cost_state = CostSectionState::Unknown;
        plan.fixed_cost_state = CostSectionState::Unknown;
        let revenue = revenue::calculate(&plan);
        assert_eq!(
            calculate(&plan, &revenue).total_cost,
            Some(Decimal::from(811_275))
        );
        assert_eq!(
            plan.effective_variable_cost_state(),
            CostSectionState::EnteredItems
        );

        plan.variable_costs.clear();
        plan.variable_cost_state = CostSectionState::EnteredItems;
        assert_eq!(
            plan.effective_variable_cost_state(),
            CostSectionState::Unknown,
            "a stale entered-items claim without rows reads as unknown, not confirmed none"
        );
    }

    #[test]
    fn a_total_only_line_contributes_its_total_and_no_per_unit_figure() {
        let mut plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let before = calculate(&plan, &revenue).variable_cost.unwrap();
        plan.variable_costs.push(VariableCostLine {
            name: "ค่าจ้างคนช่วยที่จำได้แต่ยอดรวม".into(),
            kind: VariableCostKind::HarvestLabor,
            quantity: None,
            unit: String::new(),
            unit_price: None,
            total_amount: Some(Decimal::from(12_000)),
        });

        let result = calculate(&plan, &revenue);
        let line = result.variable_lines.last().unwrap();
        assert_eq!(line.total, Some(Decimal::from(12_000)));
        assert_eq!(
            line.quantity, None,
            "a total-only harvest line does not take the sellable-yield default"
        );
        assert_eq!(line.yield_per_unit, None);
        assert_eq!(result.variable_cost, Some(before + Decimal::from(12_000)));
        assert_eq!(
            quantity_for_kind(&plan, &revenue, VariableCostKind::HarvestLabor),
            None,
            "a kind with a total-only line yields no per-unit efficiency figure"
        );
    }

    #[test]
    fn unclassified_expenses_enter_no_total() {
        let mut plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let before = calculate(&plan, &revenue);
        plan.unclassified_expenses.push(UnclassifiedExpense {
            name: "ค่าอะไรสักอย่างเดือนสาม".into(),
            amount: Some(Decimal::from(50_000)),
            note: String::new(),
        });
        let after = calculate(&plan, &revenue);
        assert_eq!(after.variable_cost, before.variable_cost);
        assert_eq!(after.fixed_cost, before.fixed_cost);
        assert_eq!(after.total_cost, before.total_cost);
    }
}
