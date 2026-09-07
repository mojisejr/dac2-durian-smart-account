use calc::{
    CheckKind, CheckStatus, HealthStatus, KpiKind, KpiVerdict, Readiness, analyze, workbook_sample,
};
use rust_decimal::Decimal;

const TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 10);

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).expect("valid golden decimal")
}

fn value(value: Option<Decimal>) -> Decimal {
    value.expect("workbook sample supplies this result")
}

fn assert_close(actual: Option<Decimal>, expected: &str) {
    let actual = value(actual);
    let expected = decimal(expected);
    assert!(
        (actual - expected).abs() <= TOLERANCE,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn every_workbook_result_used_by_the_release_matches_the_cached_fixture() {
    let result = analyze(&workbook_sample());
    assert!(result.input_issues.is_empty());

    assert_close(result.revenue.gross_yield_kg, "21000");
    assert_close(result.revenue.sellable_yield_kg, "19950");
    assert_close(result.revenue.market_gap_kg, "-5050");
    assert_close(result.revenue.grade_share_total, "1");
    assert_close(result.revenue.weighted_price_per_kg, "82.5");
    assert_close(result.revenue.revenue, "1645875");
    assert_close(result.revenue.market_fulfillment, "0.798");

    let expected_line_totals = [
        "100000", "100000", "20000", "60000", "50400", "36000", "59850", "19950", "9975", "100000",
        "0", "0", "0", "0", "0", "0",
    ];
    assert_eq!(result.cost.variable_lines.len(), expected_line_totals.len());
    for (line, expected) in result.cost.variable_lines.iter().zip(expected_line_totals) {
        assert_close(line.total, expected);
    }
    assert_close(result.cost.variable_lines[0].yield_per_unit, "3.99");
    assert_close(result.cost.variable_lines[2].yield_per_unit, "7.98");
    assert_close(result.cost.variable_lines[3].yield_per_unit, "199.5");
    assert_close(result.cost.variable_lines[4].yield_per_unit, "1.6625");
    assert_close(result.cost.variable_lines[6].quantity, "19950");
    assert_close(result.cost.variable_lines[7].quantity, "19950");
    assert_close(result.cost.variable_lines[8].quantity, "19950");
    assert_close(result.cost.variable_cost, "556175");
    assert_close(result.cost.variable_cost_per_kg, "27.87844611528822");
    assert_close(result.cost.fixed_cost, "255100");
    assert_close(result.cost.cash_fixed_cost, "192000");
    assert_close(result.cost.investment_base, "700000");
    assert_close(result.cost.total_cost, "811275");
    assert_close(result.cost.cost_per_kg, "40.665413533834588");
    assert_close(result.cost.cost_per_rai, "81127.5");

    assert_close(result.business.net_profit, "834600");
    assert_close(result.business.net_margin, "0.50708589655958081");
    assert_close(result.business.contribution_per_kg, "54.62155388471178");
    assert_close(result.business.break_even_kg, "4670.317518583096");
    assert_close(result.business.break_even_kg_per_rai, "467.03175185830958");
    assert_close(result.business.break_even_revenue, "385301.19528310542");
    assert_close(result.business.safety_margin_kg, "15279.682481416905");
    assert_close(result.business.roi, "1.1922857142857144");
    assert_close(result.business.operating_cash_flow, "897700");
    assert_close(result.business.payback_years, "0.7797705246741673");

    let expected_kpis = [
        (KpiKind::YieldPerRai, "1995", "2200", KpiVerdict::Improve),
        (KpiKind::YieldPerTree, "99.75", "110", KpiVerdict::Improve),
        (KpiKind::YieldPerLaborDay, "199.5", "180", KpiVerdict::Met),
        (
            KpiKind::YieldPerFertilizerKg,
            "3.99",
            "4.5",
            KpiVerdict::Improve,
        ),
        (
            KpiKind::YieldPerWaterCubicMeter,
            "7.98",
            "8",
            KpiVerdict::Improve,
        ),
        (KpiKind::YieldPerKwh, "1.6625", "1.8", KpiVerdict::Improve),
        (KpiKind::QualityGradeShare, "0.8", "0.8", KpiVerdict::Met),
        (KpiKind::LossShare, "0.05", "0.05", KpiVerdict::Met),
        (
            KpiKind::CostPerKg,
            "40.665413533834588",
            "45",
            KpiVerdict::Met,
        ),
    ];
    assert_eq!(result.efficiency.len(), expected_kpis.len());
    for (kpi, (kind, expected, target, verdict)) in result.efficiency.iter().zip(expected_kpis) {
        assert_eq!(kpi.kind, kind);
        assert_close(kpi.actual, expected);
        assert_close(kpi.target, target);
        assert_eq!(kpi.verdict, Some(verdict));
    }

    assert_close(result.tax.actual_expense.income, "1645875");
    assert_close(result.tax.actual_expense.expense, "811275");
    assert_close(result.tax.actual_expense.taxable_income, "774600");
    assert_close(result.tax.actual_expense.estimated_tax, "69920");
    assert_close(
        result.tax.actual_expense.average_tax_rate,
        "0.042481962481962482",
    );
    assert_close(result.tax.flat_sixty_percent.income, "1645875");
    assert_close(result.tax.flat_sixty_percent.expense, "987525");
    assert_close(result.tax.flat_sixty_percent.taxable_income, "598350");
    assert_close(result.tax.flat_sixty_percent.estimated_tax, "42252.5");
    assert_close(
        result.tax.flat_sixty_percent.average_tax_rate,
        "0.025671755145439356",
    );

    let expected_scenarios = [
        ["-121718.75", "207456.25", "289750", "372043.75", "454337.5"],
        ["-15013.75", "577501.25", "725630", "873758.75", "1021887.5"],
        ["11662.5", "670012.5", "834600", "999187.5", "1163775"],
        ["38338.75", "762523.75", "943570", "1124616.25", "1305662.5"],
        ["65015", "855035", "1052540", "1250045", "1447550"],
    ];
    for (actual_row, expected_row) in result.scenario.profits.iter().zip(expected_scenarios) {
        for (actual, expected) in actual_row.iter().copied().zip(expected_row) {
            assert_close(actual, expected);
        }
    }

    assert_eq!(result.health.dimensions.len(), 6);
    for dimension in &result.health.dimensions {
        assert_close(dimension.average, "3");
        assert_eq!(dimension.status, Some(HealthStatus::Watch));
    }
    assert_close(result.health.overall_score, "3");
    assert_eq!(result.health.overall_status, Some(HealthStatus::Watch));

    assert_eq!(result.checks.checks.len(), 6);
    assert!(
        result
            .checks
            .checks
            .iter()
            .all(|check| check.status == CheckStatus::Ok)
    );
    let expected_checks = [
        (CheckKind::GradeSharesTotalOne, "1", "1", "0"),
        (CheckKind::SellableYieldPositive, "19950", "0", "19950"),
        (
            CheckKind::PriceAboveVariableCost,
            "54.62155388471178",
            "0",
            "54.62155388471178",
        ),
        (CheckKind::InvestmentPositive, "700000", "0", "700000"),
        (CheckKind::TotalCostLinked, "811275", "811275", "0"),
        (CheckKind::HealthAnswersComplete, "12", "12", "0"),
    ];
    for (check, (kind, actual, expected, difference)) in
        result.checks.checks.iter().zip(expected_checks)
    {
        assert_eq!(check.kind, kind);
        assert_close(check.actual, actual);
        assert_close(check.expected, expected);
        assert_close(check.difference, difference);
    }
    assert_eq!(result.checks.overall, Readiness::Ready);
}
