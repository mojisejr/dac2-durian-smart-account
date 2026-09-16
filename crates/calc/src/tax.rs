use rust_decimal::Decimal;

use crate::{CostAnalysis, RevenueAnalysis, TaxAnalysis, TaxDeductionLine, TaxMethodAnalysis};

const FLAT_EXPENSE_SHARE: Decimal = Decimal::from_parts(6, 0, 0, false, 1);

/// Both methods subtract the same deductions - the owner's own lines, summed.
/// Until the owner enters any, the total is zero and the estimate is the
/// honest upper bound; the screen states that rather than assuming a figure.
pub fn calculate(
    revenue: &RevenueAnalysis,
    cost: &CostAnalysis,
    deductions: &[TaxDeductionLine],
) -> TaxAnalysis {
    let income = revenue.revenue;
    let deduction_total: Decimal = deductions.iter().map(|line| line.amount).sum();
    let actual_expense = method(income, cost.total_cost, deduction_total);
    let flat_sixty_percent = method(
        income,
        income.map(|income| income * FLAT_EXPENSE_SHARE),
        deduction_total,
    );

    TaxAnalysis {
        actual_expense,
        flat_sixty_percent,
        deduction_count: deductions.len(),
        deduction_total,
    }
}

fn method(
    income: Option<Decimal>,
    expense: Option<Decimal>,
    deductions: Decimal,
) -> TaxMethodAnalysis {
    let after_expense = income
        .zip(expense)
        .map(|(income, expense)| income - expense);
    let taxable_income = after_expense.map(|after| (after - deductions).max(Decimal::ZERO));
    let deductions_exceed_income =
        after_expense.is_some_and(|after| after > Decimal::ZERO && deductions > after);
    let estimated_tax = taxable_income.map(tax_due);
    let average_tax_rate = estimated_tax
        .zip(income)
        .and_then(|(tax, income)| (!income.is_zero()).then(|| tax / income));

    TaxMethodAnalysis {
        income,
        expense,
        deductions,
        taxable_income,
        estimated_tax,
        average_tax_rate,
        deductions_exceed_income,
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

    const PERSONAL_ALLOWANCE: Decimal = Decimal::from_parts(60_000, 0, 0, false, 0);

    fn personal_allowance() -> Vec<TaxDeductionLine> {
        vec![TaxDeductionLine {
            name: "ค่าลดหย่อนส่วนตัว".into(),
            amount: PERSONAL_ALLOWANCE,
        }]
    }

    fn revenue_of(income: Decimal) -> RevenueAnalysis {
        RevenueAnalysis {
            gross_yield_kg: None,
            sellable_yield_kg: None,
            market_gap_kg: None,
            grade_share_total: None,
            weighted_price_per_kg: None,
            revenue: Some(income),
            market_fulfillment: None,
        }
    }

    fn cost_of(total_cost: Decimal) -> CostAnalysis {
        CostAnalysis {
            variable_lines: Vec::new(),
            area_rai: None,
            variable_cost: None,
            variable_cost_per_kg: None,
            manual_fixed_cost: None,
            asset_depreciation: None,
            fixed_cost: None,
            cash_fixed_cost: None,
            manual_investment_base: None,
            asset_investment_base: None,
            starting_capital: None,
            investment_base: None,
            total_cost: Some(total_cost),
            cost_per_kg: None,
            cost_per_rai: None,
        }
    }

    fn analysis_for(income: Decimal, actual_expense: Decimal) -> TaxAnalysis {
        calculate(
            &revenue_of(income),
            &cost_of(actual_expense),
            &personal_allowance(),
        )
    }

    #[test]
    fn workbook_tax_methods_match() {
        let plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let cost = cost::calculate(&plan, &revenue);
        let result = calculate(&revenue, &cost, &plan.tax_deductions);
        assert_eq!(result.deduction_count, 1);
        assert_eq!(result.deduction_total, PERSONAL_ALLOWANCE);

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
        let result = method(Some(Decimal::ZERO), Some(Decimal::ZERO), Decimal::ZERO);
        assert_eq!(result.estimated_tax, Some(Decimal::ZERO));
        assert_eq!(result.average_tax_rate, None);
    }

    #[test]
    fn no_lines_deduct_nothing_and_say_so() {
        let plan = workbook_sample();
        let revenue = revenue::calculate(&plan);
        let cost = cost::calculate(&plan, &revenue);
        let result = calculate(&revenue, &cost, &[]);
        assert_eq!(result.deduction_count, 0);
        assert_eq!(result.deduction_total, Decimal::ZERO);
        // 1,645,875 - 811,275 with nothing deducted: 60,000 more than the
        // workbook's taxable income under the actual method.
        assert_eq!(
            result.actual_expense.taxable_income,
            Some(Decimal::from(834_600))
        );
        assert!(!result.actual_expense.deductions_exceed_income);
    }

    #[test]
    fn several_lines_sum_and_apply_to_both_methods_equally() {
        let lines = vec![
            TaxDeductionLine {
                name: "ค่าลดหย่อนส่วนตัว".into(),
                amount: Decimal::from(60_000),
            },
            TaxDeductionLine {
                name: "ประกันสังคม".into(),
                amount: Decimal::from(9_000),
            },
            TaxDeductionLine {
                name: "ประกันชีวิต".into(),
                amount: Decimal::from(91_000),
            },
        ];
        let result = calculate(
            &revenue_of(Decimal::from(1_000_000)),
            &cost_of(Decimal::from(300_000)),
            &lines,
        );
        assert_eq!(result.deduction_count, 3);
        assert_eq!(result.deduction_total, Decimal::from(160_000));
        // actual: 1,000,000 - 300,000 - 160,000
        assert_eq!(
            result.actual_expense.taxable_income,
            Some(Decimal::from(540_000))
        );
        // flat: 1,000,000 - 600,000 - 160,000
        assert_eq!(
            result.flat_sixty_percent.taxable_income,
            Some(Decimal::from(240_000))
        );
        assert_eq!(result.actual_expense.deductions, Decimal::from(160_000));
        assert_eq!(result.flat_sixty_percent.deductions, Decimal::from(160_000));
    }

    #[test]
    fn deductions_above_income_after_expense_floor_at_zero_and_are_flagged() {
        let lines = vec![TaxDeductionLine {
            name: "รวมทุกอย่าง".into(),
            amount: Decimal::from(500_000),
        }];
        let result = calculate(
            &revenue_of(Decimal::from(1_000_000)),
            &cost_of(Decimal::from(700_000)),
            &lines,
        );
        // actual: 300,000 after expense, 500,000 deducted
        assert_eq!(result.actual_expense.taxable_income, Some(Decimal::ZERO));
        assert_eq!(result.actual_expense.estimated_tax, Some(Decimal::ZERO));
        assert!(result.actual_expense.deductions_exceed_income);
        // flat: 400,000 after expense, 500,000 deducted
        assert_eq!(
            result.flat_sixty_percent.taxable_income,
            Some(Decimal::ZERO)
        );
        assert!(result.flat_sixty_percent.deductions_exceed_income);
        // A loss before deductions is not "deductions exceed income".
        let loss = calculate(
            &revenue_of(Decimal::from(100_000)),
            &cost_of(Decimal::from(700_000)),
            &lines,
        );
        assert_eq!(loss.actual_expense.taxable_income, Some(Decimal::ZERO));
        assert!(!loss.actual_expense.deductions_exceed_income);
    }

    #[test]
    fn a_zero_line_counts_as_entered() {
        let lines = vec![TaxDeductionLine {
            name: "บริจาค".into(),
            amount: Decimal::ZERO,
        }];
        let result = calculate(
            &revenue_of(Decimal::from(1_000_000)),
            &cost_of(Decimal::ZERO),
            &lines,
        );
        assert_eq!(result.deduction_count, 1);
        assert_eq!(result.deduction_total, Decimal::ZERO);
    }
}
