use rust_decimal::Decimal;

use crate::{Plan, RevenueAnalysis};

pub fn calculate(plan: &Plan) -> RevenueAnalysis {
    let production = &plan.production;
    let gross_yield_kg = production.derived_gross_yield_kg();
    let sellable_yield_kg = production.selected_sellable_yield_kg();
    let market_gap_kg = sellable_yield_kg
        .zip(plan.market.buyer_committed_kg)
        .map(|(sellable, committed)| sellable - committed);
    let grade_share_total = complete_sum(plan.production.grades.iter().map(|grade| grade.share));
    let weighted_price_per_kg = production.selected_price_per_kg();
    let revenue = sellable_yield_kg
        .zip(weighted_price_per_kg)
        .map(|(yield_kg, price)| yield_kg * price);
    let market_fulfillment = sellable_yield_kg
        .zip(plan.market.buyer_committed_kg)
        .and_then(|(sellable, committed)| nonzero_ratio(sellable, committed));

    RevenueAnalysis {
        gross_yield_kg,
        sellable_yield_kg,
        market_gap_kg,
        grade_share_total,
        weighted_price_per_kg,
        revenue,
        market_fulfillment,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Grade, PriceSource, YieldSource, workbook_sample};

    #[test]
    fn workbook_revenue_matches_the_four_grade_sheet() {
        let result = calculate(&workbook_sample());

        assert_eq!(result.gross_yield_kg, Some(Decimal::from(21_000)));
        assert_eq!(result.sellable_yield_kg, Some(Decimal::from(19_950)));
        assert_eq!(result.grade_share_total, Some(Decimal::ONE));
        assert_eq!(result.weighted_price_per_kg, Some(Decimal::new(825, 1)));
        assert_eq!(result.revenue, Some(Decimal::from(1_645_875)));
    }

    #[test]
    fn zero_and_missing_buyer_commitment_produce_no_ratio() {
        let mut plan = workbook_sample();
        plan.market.buyer_committed_kg = Some(Decimal::ZERO);
        assert_eq!(calculate(&plan).market_fulfillment, None);

        plan.market.buyer_committed_kg = None;
        assert_eq!(calculate(&plan).market_fulfillment, None);
    }

    #[test]
    fn zero_one_and_ten_grade_lists_have_defined_results() {
        let mut plan = workbook_sample();
        plan.production.grades.clear();
        let empty = calculate(&plan);
        assert_eq!(empty.grade_share_total, None);
        assert_eq!(empty.weighted_price_per_kg, None);

        plan.production.grades.push(Grade {
            name: "ทั้งหมด".into(),
            share: Some(Decimal::ONE),
            price_per_kg: Some(Decimal::from(80)),
            counts_as_quality_grade: true,
        });
        assert_eq!(
            calculate(&plan).weighted_price_per_kg,
            Some(Decimal::from(80))
        );

        plan.production.grades = (1..=10)
            .map(|index| Grade {
                name: format!("เกรด {index}"),
                share: Some(Decimal::new(1, 1)),
                price_per_kg: Some(Decimal::from(index * 10)),
                counts_as_quality_grade: index <= 2,
            })
            .collect();
        let ten = calculate(&plan);
        assert_eq!(ten.grade_share_total, Some(Decimal::ONE));
        assert_eq!(ten.weighted_price_per_kg, Some(Decimal::from(55)));
    }

    #[test]
    fn a_missing_grade_value_keeps_weighted_price_unavailable() {
        let mut plan = workbook_sample();
        plan.production.grades[0].price_per_kg = None;
        assert_eq!(calculate(&plan).weighted_price_per_kg, None);
    }

    #[test]
    fn direct_and_derived_yield_agree_on_an_equivalent_fixture() {
        let derived = workbook_sample();
        let mut direct = workbook_sample();
        direct.production.yield_source = YieldSource::Direct;
        direct.production.sellable_yield_kg = Some(Decimal::from(19_950));

        let derived = calculate(&derived);
        let direct = calculate(&direct);
        assert_eq!(direct.sellable_yield_kg, derived.sellable_yield_kg);
        assert_eq!(direct.revenue, derived.revenue);
        assert_eq!(direct.market_fulfillment, derived.market_fulfillment);
    }

    #[test]
    fn only_the_selected_yield_branch_feeds_sellable_kilograms() {
        let mut plan = workbook_sample();
        plan.production.sellable_yield_kg = Some(Decimal::from(1));
        assert_eq!(
            calculate(&plan).sellable_yield_kg,
            Some(Decimal::from(19_950)),
            "a stored direct figure must not leak into the derived branch"
        );

        plan.production.yield_source = YieldSource::Direct;
        assert_eq!(calculate(&plan).sellable_yield_kg, Some(Decimal::from(1)));
        assert_eq!(
            calculate(&plan).gross_yield_kg,
            Some(Decimal::from(21_000)),
            "the derived facts stay available beside the direct entry"
        );

        plan.production.sellable_yield_kg = None;
        assert_eq!(
            calculate(&plan).sellable_yield_kg,
            None,
            "a direct branch without its figure never falls back to the derived one"
        );
    }

    #[test]
    fn average_and_by_grade_price_agree_on_an_equivalent_fixture() {
        let by_grade = workbook_sample();
        let mut average = workbook_sample();
        average.production.price_source = PriceSource::Average;
        average.production.average_price_per_kg = Some(Decimal::new(825, 1));

        assert_eq!(
            calculate(&average).weighted_price_per_kg,
            calculate(&by_grade).weighted_price_per_kg
        );
        assert_eq!(calculate(&average).revenue, calculate(&by_grade).revenue);
    }

    #[test]
    fn only_the_selected_price_branch_feeds_revenue_and_the_other_is_kept() {
        let mut plan = workbook_sample();
        plan.production.average_price_per_kg = Some(Decimal::from(1));
        assert_eq!(
            calculate(&plan).weighted_price_per_kg,
            Some(Decimal::new(825, 1)),
            "a stored average must not leak into the by-grade branch"
        );

        plan.production.price_source = PriceSource::Average;
        assert_eq!(
            calculate(&plan).weighted_price_per_kg,
            Some(Decimal::from(1))
        );
        assert_eq!(
            plan.production.weighted_grade_price_per_kg(),
            Some(Decimal::new(825, 1)),
            "grade facts stay available while the average branch is selected"
        );

        plan.production.average_price_per_kg = None;
        assert_eq!(
            calculate(&plan).weighted_price_per_kg,
            None,
            "an average branch without its figure never falls back to grades"
        );
    }
}
