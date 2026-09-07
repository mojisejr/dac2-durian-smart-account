use rust_decimal::Decimal;

use crate::{Plan, RevenueAnalysis};

pub fn calculate(plan: &Plan) -> RevenueAnalysis {
    let production = &plan.production;
    let gross_yield_kg = multiply([
        production.producing_trees,
        production.fruits_per_tree,
        production.average_fruit_weight_kg,
    ]);
    let sellable_yield_kg = gross_yield_kg
        .zip(production.loss_share)
        .map(|(gross, loss)| gross * (Decimal::ONE - loss));
    let market_gap_kg = sellable_yield_kg
        .zip(plan.market.demand_kg)
        .map(|(sellable, demand)| sellable - demand);
    let grade_share_total = complete_sum(plan.production.grades.iter().map(|grade| grade.share));
    let weighted_price_per_kg = complete_sum(plan.production.grades.iter().map(|grade| {
        grade
            .share
            .zip(grade.price_per_kg)
            .map(|(share, price)| share * price)
    }));
    let revenue = sellable_yield_kg
        .zip(weighted_price_per_kg)
        .map(|(yield_kg, price)| yield_kg * price);
    let market_fulfillment = sellable_yield_kg
        .zip(plan.market.demand_kg)
        .and_then(|(sellable, demand)| nonzero_ratio(sellable, demand));

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

fn multiply<const N: usize>(values: [Option<Decimal>; N]) -> Option<Decimal> {
    values.into_iter().try_fold(Decimal::ONE, |product, value| {
        value.map(|value| product * value)
    })
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
    use crate::{Grade, workbook_sample};

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
    fn zero_and_missing_market_demand_produce_no_ratio() {
        let mut plan = workbook_sample();
        plan.market.demand_kg = Some(Decimal::ZERO);
        assert_eq!(calculate(&plan).market_fulfillment, None);

        plan.market.demand_kg = None;
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
}
