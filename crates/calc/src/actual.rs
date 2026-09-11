use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActualOutcome {
    pub sellable_yield_kg: Option<Decimal>,
    pub revenue: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub note: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActualInputField {
    SellableYieldKg,
    Revenue,
    TotalCost,
}

impl ActualInputField {
    pub const ALL: [Self; 3] = [Self::SellableYieldKg, Self::Revenue, Self::TotalCost];
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OutcomeMetrics {
    pub sellable_yield_kg: Option<Decimal>,
    pub revenue: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub profit: Option<Decimal>,
    pub average_price_per_kg: Option<Decimal>,
    pub cost_per_kg: Option<Decimal>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActualAnalysis {
    pub input_issues: Vec<ActualInputField>,
    pub metrics: OutcomeMetrics,
}

pub fn analyze_actual(outcome: &ActualOutcome) -> ActualAnalysis {
    let input_issues = ActualInputField::ALL
        .into_iter()
        .filter(|field| value(outcome, *field).is_none_or(|value| value < Decimal::ZERO))
        .collect::<Vec<_>>();

    if !input_issues.is_empty() {
        return ActualAnalysis {
            input_issues,
            metrics: OutcomeMetrics::default(),
        };
    }

    let sellable_yield_kg = outcome.sellable_yield_kg;
    let revenue = outcome.revenue;
    let total_cost = outcome.total_cost;
    let profit = revenue
        .zip(total_cost)
        .map(|(revenue, cost)| revenue - cost);
    let average_price_per_kg = revenue
        .zip(sellable_yield_kg)
        .and_then(|(revenue, yield_kg)| nonzero_ratio(revenue, yield_kg));
    let cost_per_kg = total_cost
        .zip(sellable_yield_kg)
        .and_then(|(cost, yield_kg)| nonzero_ratio(cost, yield_kg));

    ActualAnalysis {
        input_issues,
        metrics: OutcomeMetrics {
            sellable_yield_kg,
            revenue,
            total_cost,
            profit,
            average_price_per_kg,
            cost_per_kg,
        },
    }
}

fn value(outcome: &ActualOutcome, field: ActualInputField) -> Option<Decimal> {
    match field {
        ActualInputField::SellableYieldKg => outcome.sellable_yield_kg,
        ActualInputField::Revenue => outcome.revenue,
        ActualInputField::TotalCost => outcome.total_cost,
    }
}

fn nonzero_ratio(numerator: Decimal, denominator: Decimal) -> Option<Decimal> {
    (!denominator.is_zero()).then(|| numerator / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(yield_kg: i64, revenue: i64, cost: i64) -> ActualOutcome {
        ActualOutcome {
            sellable_yield_kg: Some(Decimal::from(yield_kg)),
            revenue: Some(Decimal::from(revenue)),
            total_cost: Some(Decimal::from(cost)),
            note: "ฝนมากกว่าคาด".into(),
        }
    }

    #[test]
    fn derives_actual_profit_price_and_cost_per_kg() {
        let analysis = analyze_actual(&outcome(18_000, 1_530_000, 990_000));

        assert!(analysis.input_issues.is_empty());
        assert_eq!(analysis.metrics.profit, Some(Decimal::from(540_000)));
        assert_eq!(
            analysis.metrics.average_price_per_kg,
            Some(Decimal::from(85))
        );
        assert_eq!(analysis.metrics.cost_per_kg, Some(Decimal::from(55)));
    }

    #[test]
    fn zero_yield_is_a_real_outcome_but_ratios_are_unavailable() {
        let analysis = analyze_actual(&outcome(0, 0, 300_000));

        assert!(analysis.input_issues.is_empty());
        assert_eq!(analysis.metrics.profit, Some(Decimal::from(-300_000)));
        assert_eq!(analysis.metrics.average_price_per_kg, None);
        assert_eq!(analysis.metrics.cost_per_kg, None);
    }

    #[test]
    fn missing_or_negative_facts_withhold_all_derived_results() {
        let missing = analyze_actual(&ActualOutcome::default());
        assert_eq!(missing.input_issues, ActualInputField::ALL);
        assert_eq!(missing.metrics, OutcomeMetrics::default());

        let negative = analyze_actual(&outcome(10, -1, 5));
        assert_eq!(negative.input_issues, vec![ActualInputField::Revenue]);
        assert_eq!(negative.metrics, OutcomeMetrics::default());
    }
}
