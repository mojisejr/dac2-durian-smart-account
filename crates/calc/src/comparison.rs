use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{ActualOutcome, ForecastMode, OutcomeMetrics, Plan, QuickEstimate};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ComparisonMetric {
    SellableYieldKg,
    Revenue,
    TotalCost,
    Profit,
    AveragePricePerKg,
    CostPerKg,
}

impl ComparisonMetric {
    pub const ALL: [Self; 6] = [
        Self::SellableYieldKg,
        Self::Revenue,
        Self::TotalCost,
        Self::Profit,
        Self::AveragePricePerKg,
        Self::CostPerKg,
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric: ComparisonMetric,
    pub forecast: Option<Decimal>,
    pub actual: Option<Decimal>,
    pub delta: Option<Decimal>,
}

pub fn forecast_metrics(
    mode: ForecastMode,
    quick: &QuickEstimate,
    detailed: &Plan,
) -> OutcomeMetrics {
    forecast_metrics_with_assets(mode, quick, detailed, &[], None)
}

pub fn forecast_metrics_with_assets(
    mode: ForecastMode,
    quick: &QuickEstimate,
    detailed: &Plan,
    assets: &[crate::AssetAllocation],
    starting_capital: Option<Decimal>,
) -> OutcomeMetrics {
    match mode {
        ForecastMode::Quick => {
            let analysis = crate::analyze_quick(quick);
            OutcomeMetrics {
                sellable_yield_kg: quick.sellable_yield_kg,
                revenue: analysis.revenue,
                total_cost: quick.total_cost,
                profit: analysis.net_profit,
                average_price_per_kg: quick.average_price_per_kg,
                cost_per_kg: analysis.cost_per_kg,
            }
        }
        ForecastMode::Detailed => {
            let analysis = crate::analyze_with_assets(detailed, assets, starting_capital);
            OutcomeMetrics {
                sellable_yield_kg: analysis.revenue.sellable_yield_kg,
                revenue: analysis.revenue.revenue,
                total_cost: analysis.cost.total_cost,
                profit: analysis.business.net_profit,
                average_price_per_kg: analysis.revenue.weighted_price_per_kg,
                cost_per_kg: analysis.cost.cost_per_kg,
            }
        }
    }
}

pub fn compare(forecast: &OutcomeMetrics, actual: &ActualOutcome) -> Vec<MetricComparison> {
    let actual = crate::analyze_actual(actual).metrics;
    compare_metrics(forecast, &actual)
}

pub fn compare_metrics(
    forecast: &OutcomeMetrics,
    actual: &OutcomeMetrics,
) -> Vec<MetricComparison> {
    ComparisonMetric::ALL
        .into_iter()
        .map(|metric| {
            let forecast_value = metric_value(forecast, metric);
            let actual_value = metric_value(actual, metric);
            MetricComparison {
                metric,
                forecast: forecast_value,
                actual: actual_value,
                delta: actual_value
                    .zip(forecast_value)
                    .map(|(actual, forecast)| actual - forecast),
            }
        })
        .collect()
}

pub(crate) fn metric_value(metrics: &OutcomeMetrics, metric: ComparisonMetric) -> Option<Decimal> {
    match metric {
        ComparisonMetric::SellableYieldKg => metrics.sellable_yield_kg,
        ComparisonMetric::Revenue => metrics.revenue,
        ComparisonMetric::TotalCost => metrics.total_cost,
        ComparisonMetric::Profit => metrics.profit,
        ComparisonMetric::AveragePricePerKg => metrics.average_price_per_kg,
        ComparisonMetric::CostPerKg => metrics.cost_per_kg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workbook_sample;

    #[test]
    fn quick_forecast_snapshot_uses_only_the_three_quick_facts() {
        let quick = QuickEstimate {
            sellable_yield_kg: Some(Decimal::from(20_000)),
            average_price_per_kg: Some(Decimal::from(80)),
            total_cost: Some(Decimal::from(900_000)),
        };

        let metrics = forecast_metrics(ForecastMode::Quick, &quick, &workbook_sample());
        assert_eq!(metrics.revenue, Some(Decimal::from(1_600_000)));
        assert_eq!(metrics.profit, Some(Decimal::from(700_000)));
        assert_eq!(metrics.cost_per_kg, Some(Decimal::from(45)));
    }

    #[test]
    fn detailed_forecast_snapshot_matches_existing_analysis() {
        let plan = workbook_sample();
        let expected = crate::analyze(&plan);
        let metrics = forecast_metrics(ForecastMode::Detailed, &QuickEstimate::default(), &plan);

        assert_eq!(
            metrics.sellable_yield_kg,
            expected.revenue.sellable_yield_kg
        );
        assert_eq!(metrics.revenue, expected.revenue.revenue);
        assert_eq!(metrics.total_cost, expected.cost.total_cost);
        assert_eq!(metrics.profit, expected.business.net_profit);
        assert_eq!(
            metrics.average_price_per_kg,
            expected.revenue.weighted_price_per_kg
        );
        assert_eq!(metrics.cost_per_kg, expected.cost.cost_per_kg);
    }

    #[test]
    fn comparison_exposes_forecast_actual_and_actual_minus_forecast() {
        let forecast = OutcomeMetrics {
            sellable_yield_kg: Some(Decimal::from(20_000)),
            revenue: Some(Decimal::from(1_600_000)),
            total_cost: Some(Decimal::from(900_000)),
            profit: Some(Decimal::from(700_000)),
            average_price_per_kg: Some(Decimal::from(80)),
            cost_per_kg: Some(Decimal::from(45)),
        };
        let actual = ActualOutcome {
            sellable_yield_kg: Some(Decimal::from(18_000)),
            revenue: Some(Decimal::from(1_530_000)),
            total_cost: Some(Decimal::from(990_000)),
            note: String::new(),
        };

        let result = compare(&forecast, &actual);
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].delta, Some(Decimal::from(-2_000)));
        assert_eq!(result[1].delta, Some(Decimal::from(-70_000)));
        assert_eq!(result[2].delta, Some(Decimal::from(90_000)));
        assert_eq!(result[3].delta, Some(Decimal::from(-160_000)));
        assert_eq!(result[4].delta, Some(Decimal::from(5)));
        assert_eq!(result[5].delta, Some(Decimal::from(10)));
    }

    #[test]
    fn comparison_keeps_unavailable_values_explicit() {
        let result = compare(
            &OutcomeMetrics::default(),
            &ActualOutcome {
                sellable_yield_kg: Some(Decimal::ZERO),
                revenue: Some(Decimal::ZERO),
                total_cost: Some(Decimal::from(10)),
                note: String::new(),
            },
        );

        assert!(result.iter().all(|row| row.delta.is_none()));
        assert_eq!(
            result
                .iter()
                .find(|row| row.metric == ComparisonMetric::CostPerKg)
                .and_then(|row| row.actual),
            None
        );
    }
}
