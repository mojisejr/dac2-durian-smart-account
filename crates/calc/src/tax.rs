use rust_decimal::Decimal;

use crate::{CostAnalysis, RevenueAnalysis, TaxAnalysis, TaxMethodAnalysis};

pub const PERSONAL_ALLOWANCE: Decimal = Decimal::from_parts(60_000, 0, 0, false, 0);
const FLAT_EXPENSE_SHARE: Decimal = Decimal::from_parts(6, 0, 0, false, 1);

pub fn calculate(revenue: &RevenueAnalysis, cost: &CostAnalysis) -> TaxAnalysis {
    let income = revenue.revenue;
    let actual_expense = method(income, cost.total_cost);
    let flat_sixty_percent = method(income, income.map(|income| income * FLAT_EXPENSE_SHARE));

    TaxAnalysis {
        actual_expense,
        flat_sixty_percent,
    }
}

fn method(income: Option<Decimal>, expense: Option<Decimal>) -> TaxMethodAnalysis {
    let taxable_income = income
        .zip(expense)
        .map(|(income, expense)| (income - expense - PERSONAL_ALLOWANCE).max(Decimal::ZERO));
    let estimated_tax = taxable_income.map(tax_due);
    let average_tax_rate = estimated_tax
        .zip(income)
        .and_then(|(tax, income)| (!income.is_zero()).then(|| tax / income));

    TaxMethodAnalysis {
        income,
        expense,
        personal_allowance: PERSONAL_ALLOWANCE,
        taxable_income,
        estimated_tax,
        average_tax_rate,
    }
}

pub fn tax_due(taxable_income: Decimal) -> Decimal {
    let taxable_income = taxable_income.max(Decimal::ZERO);
    if taxable_income <= Decimal::from(150_000) {
        Decimal::ZERO
    } else if taxable_income <= Decimal::from(300_000) {
        (taxable_income - Decimal::from(150_000)) * Decimal::new(5, 2)
    } else if taxable_income <= Decimal::from(500_000) {
        Decimal::from(7_500) + (taxable_income - Decimal::from(300_000)) * Decimal::new(1, 1)
    } else if taxable_income <= Decimal::from(750_000) {
        Decimal::from(27_500) + (taxable_income - Decimal::from(500_000)) * Decimal::new(15, 2)
    } else if taxable_income <= Decimal::from(1_000_000) {
        Decimal::from(65_000) + (taxable_income - Decimal::from(750_000)) * Decimal::new(2, 1)
    } else if taxable_income <= Decimal::from(2_000_000) {
        Decimal::from(115_000) + (taxable_income - Decimal::from(1_000_000)) * Decimal::new(25, 2)
    } else if taxable_income <= Decimal::from(5_000_000) {
        Decimal::from(365_000) + (taxable_income - Decimal::from(2_000_000)) * Decimal::new(3, 1)
    } else {
        Decimal::from(1_265_000) + (taxable_income - Decimal::from(5_000_000)) * Decimal::new(35, 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CostAnalysis, RevenueAnalysis, cost, revenue, workbook_sample};

    fn analysis_for(income: Decimal, actual_expense: Decimal) -> TaxAnalysis {
        let revenue = RevenueAnalysis {
            gross_yield_kg: None,
            sellable_yield_kg: None,
            market_gap_kg: None,
            grade_share_total: None,
            weighted_price_per_kg: None,
            revenue: Some(income),
            market_fulfillment: None,
        };
        let cost = CostAnalysis {
            variable_lines: Vec::new(),
            area_rai: None,
            variable_cost: None,
            variable_cost_per_kg: None,
            fixed_cost: None,
            cash_fixed_cost: None,
            investment_base: None,
            total_cost: Some(actual_expense),
            cost_per_kg: None,
            cost_per_rai: None,
        };
        calculate(&revenue, &cost)
    }

    #[test]
    fn workbook_tax_methods_match() {
        let plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let cost = cost::calculate(&plan, &revenue);
        let result = calculate(&revenue, &cost);

        assert_eq!(
            result.actual_expense.taxable_income,
            Some(Decimal::from(774_600))
        );
        assert_eq!(
            result.actual_expense.estimated_tax,
            Some(Decimal::from(69_920))
        );
        assert_eq!(
            result.flat_sixty_percent.taxable_income,
            Some(Decimal::from(598_350))
        );
        assert_eq!(
            result.flat_sixty_percent.estimated_tax,
            Some(Decimal::new(422_525, 1))
        );
    }

    #[test]
    fn every_tax_boundary_is_exact_below_at_and_above_under_both_methods() {
        let cases = [
            (150_000, "0", "0", "0.05"),
            (300_000, "7499.95", "7500", "7500.10"),
            (500_000, "27499.90", "27500", "27500.15"),
            (750_000, "64999.85", "65000", "65000.20"),
            (1_000_000, "114999.80", "115000", "115000.25"),
            (2_000_000, "364999.75", "365000", "365000.30"),
            (5_000_000, "1264999.70", "1265000", "1265000.35"),
        ];
        for (boundary, below, at, above) in cases {
            let boundary = Decimal::from(boundary);
            for (taxable_income, expected) in [
                (boundary - Decimal::ONE, below),
                (boundary, at),
                (boundary + Decimal::ONE, above),
            ] {
                let expected = Decimal::from_str_exact(expected).unwrap();

                let actual_income = Decimal::from(10_000_000);
                let actual_expense = actual_income - PERSONAL_ALLOWANCE - taxable_income;
                let actual = analysis_for(actual_income, actual_expense).actual_expense;
                assert_eq!(actual.taxable_income, Some(taxable_income));
                assert_eq!(actual.estimated_tax, Some(expected));

                let flat_income =
                    (taxable_income + PERSONAL_ALLOWANCE) / (Decimal::ONE - FLAT_EXPENSE_SHARE);
                let flat = analysis_for(flat_income, Decimal::ZERO).flat_sixty_percent;
                assert_eq!(flat.taxable_income, Some(taxable_income));
                assert_eq!(flat.estimated_tax, Some(expected));
            }
        }
        assert_eq!(tax_due(Decimal::NEGATIVE_ONE), Decimal::ZERO);
    }

    #[test]
    fn zero_income_has_no_average_rate() {
        let result = method(Some(Decimal::ZERO), Some(Decimal::ZERO));
        assert_eq!(result.estimated_tax, Some(Decimal::ZERO));
        assert_eq!(result.average_tax_rate, None);
    }
}
