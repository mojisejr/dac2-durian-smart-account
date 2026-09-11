use rust_decimal::Decimal;

use crate::{BusinessAnalysis, CostAnalysis, RevenueAnalysis};

pub fn calculate(revenue: &RevenueAnalysis, cost: &CostAnalysis) -> BusinessAnalysis {
    let net_profit = revenue
        .revenue
        .zip(cost.total_cost)
        .map(|(revenue, cost)| revenue - cost);
    let net_margin = net_profit
        .zip(revenue.revenue)
        .and_then(|(profit, revenue)| nonzero_ratio(profit, revenue));
    let contribution_per_kg = revenue
        .weighted_price_per_kg
        .zip(cost.variable_cost_per_kg)
        .map(|(price, variable_cost)| price - variable_cost);
    let break_even_kg = contribution_per_kg
        .zip(cost.fixed_cost)
        .and_then(|(contribution, fixed)| positive_ratio(fixed, contribution));
    let break_even_kg_per_rai = break_even_kg
        .zip(cost.area_rai)
        .and_then(|(break_even, area_rai)| positive_ratio(break_even, area_rai));
    let break_even_revenue = break_even_kg
        .zip(revenue.weighted_price_per_kg)
        .map(|(yield_kg, price)| yield_kg * price);
    let safety_margin_kg = revenue
        .sellable_yield_kg
        .zip(break_even_kg)
        .map(|(yield_kg, break_even)| yield_kg - break_even);
    let roi = net_profit
        .zip(cost.investment_base)
        .and_then(|(profit, investment)| nonzero_ratio(profit, investment));
    let operating_cash_flow = revenue
        .revenue
        .zip(cost.variable_cost)
        .zip(cost.cash_fixed_cost)
        .map(|((revenue, variable), fixed_cash)| revenue - variable - fixed_cash);
    let payback_years = cost
        .investment_base
        .zip(operating_cash_flow)
        .and_then(|(investment, cash_flow)| positive_ratio(investment, cash_flow));

    BusinessAnalysis {
        net_profit,
        net_margin,
        contribution_per_kg,
        break_even_kg,
        break_even_kg_per_rai,
        break_even_revenue,
        safety_margin_kg,
        roi,
        operating_cash_flow,
        payback_years,
    }
}

fn nonzero_ratio(numerator: Decimal, denominator: Decimal) -> Option<Decimal> {
    (!denominator.is_zero()).then(|| numerator / denominator)
}

fn positive_ratio(numerator: Decimal, denominator: Decimal) -> Option<Decimal> {
    (denominator > Decimal::ZERO).then(|| numerator / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cost, revenue, workbook_sample};

    #[test]
    fn workbook_business_results_match() {
        let plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let cost = cost::calculate(&plan, &revenue);
        let result = calculate(&revenue, &cost);

        assert_eq!(result.net_profit, Some(Decimal::from(834_600)));
        assert_eq!(result.operating_cash_flow, Some(Decimal::from(897_700)));
        assert_eq!(
            result.roi,
            Some(Decimal::from(834_600) / Decimal::from(700_000))
        );
        assert_eq!(
            result.payback_years,
            Some(Decimal::from(700_000) / Decimal::from(897_700))
        );
    }

    #[test]
    fn zero_or_negative_denominators_are_not_invented_as_zero() {
        assert_eq!(nonzero_ratio(Decimal::ONE, Decimal::ZERO), None);
        assert_eq!(positive_ratio(Decimal::ONE, Decimal::ZERO), None);
        assert_eq!(positive_ratio(Decimal::ONE, Decimal::NEGATIVE_ONE), None);
    }

    #[test]
    fn non_positive_contribution_and_cash_flow_have_no_break_even_or_payback() {
        let revenue = RevenueAnalysis {
            gross_yield_kg: None,
            sellable_yield_kg: None,
            market_gap_kg: None,
            grade_share_total: None,
            weighted_price_per_kg: Some(Decimal::from(10)),
            revenue: Some(Decimal::from(100)),
            market_fulfillment: None,
        };
        let cost = CostAnalysis {
            variable_lines: Vec::new(),
            area_rai: None,
            variable_cost: Some(Decimal::from(200)),
            variable_cost_per_kg: Some(Decimal::from(10)),
            manual_fixed_cost: Some(Decimal::from(50)),
            asset_depreciation: None,
            fixed_cost: Some(Decimal::from(50)),
            cash_fixed_cost: Some(Decimal::from(20)),
            manual_investment_base: Some(Decimal::from(100)),
            asset_investment_base: None,
            starting_capital: None,
            investment_base: Some(Decimal::from(100)),
            total_cost: Some(Decimal::from(250)),
            cost_per_kg: None,
            cost_per_rai: None,
        };
        let result = calculate(&revenue, &cost);

        assert_eq!(result.break_even_kg, None);
        assert_eq!(result.payback_years, None);
    }

    #[test]
    fn each_business_ratio_rejects_its_zero_denominator() {
        let revenue = RevenueAnalysis {
            gross_yield_kg: None,
            sellable_yield_kg: Some(Decimal::from(10)),
            market_gap_kg: None,
            grade_share_total: None,
            weighted_price_per_kg: Some(Decimal::from(20)),
            revenue: Some(Decimal::ZERO),
            market_fulfillment: None,
        };
        let cost = CostAnalysis {
            variable_lines: Vec::new(),
            area_rai: Some(Decimal::ZERO),
            variable_cost: Some(Decimal::from(10)),
            variable_cost_per_kg: Some(Decimal::from(5)),
            manual_fixed_cost: Some(Decimal::from(10)),
            asset_depreciation: None,
            fixed_cost: Some(Decimal::from(10)),
            cash_fixed_cost: Some(Decimal::from(5)),
            manual_investment_base: Some(Decimal::ZERO),
            asset_investment_base: None,
            starting_capital: None,
            investment_base: Some(Decimal::ZERO),
            total_cost: Some(Decimal::from(20)),
            cost_per_kg: Some(Decimal::from(2)),
            cost_per_rai: None,
        };
        let result = calculate(&revenue, &cost);

        assert_eq!(result.net_margin, None);
        assert_eq!(result.break_even_kg_per_rai, None);
        assert_eq!(result.roi, None);
    }
}
