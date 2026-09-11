use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{ComparisonMetric, MetricComparison, OutcomeMetrics, comparison::metric_value};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Higher,
    Lower,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricTrend {
    pub metric: ComparisonMetric,
    pub previous: Option<Decimal>,
    pub current: Option<Decimal>,
    pub delta: Option<Decimal>,
    pub percent_change: Option<Decimal>,
    pub direction: Option<TrendDirection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeasonSnapshot {
    pub season_id: i64,
    pub season_year: Option<i32>,
    pub actual: Option<OutcomeMetrics>,
    pub forecast: Option<OutcomeMetrics>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeasonHistoryEntry {
    pub season: SeasonSnapshot,
    pub baseline: bool,
    pub previous_actual_season_id: Option<i64>,
    pub previous_actual_year: Option<i32>,
    pub actual_trends: Vec<MetricTrend>,
    pub forecast_comparison: Vec<MetricComparison>,
}

pub fn build_season_history(mut seasons: Vec<SeasonSnapshot>) -> Vec<SeasonHistoryEntry> {
    seasons.sort_by_key(|season| {
        (
            season.season_year.is_none(),
            season.season_year.unwrap_or_default(),
            season.season_id,
        )
    });

    let mut previous_actual: Option<(i64, Option<i32>, OutcomeMetrics)> = None;
    seasons
        .into_iter()
        .map(|season| {
            let baseline = season.actual.is_some() && previous_actual.is_none();
            let (previous_actual_season_id, previous_actual_year, actual_trends) =
                match (&season.actual, &previous_actual) {
                    (Some(current), Some((id, year, previous))) => {
                        (Some(*id), *year, compare_actuals(previous, current))
                    }
                    _ => (None, None, Vec::new()),
                };
            let forecast_comparison = season
                .forecast
                .as_ref()
                .zip(season.actual.as_ref())
                .map(|(forecast, actual)| crate::compare_metrics(forecast, actual))
                .unwrap_or_default();

            if let Some(actual) = &season.actual {
                previous_actual = Some((season.season_id, season.season_year, actual.clone()));
            }

            SeasonHistoryEntry {
                season,
                baseline,
                previous_actual_season_id,
                previous_actual_year,
                actual_trends,
                forecast_comparison,
            }
        })
        .collect()
}

pub fn compare_actuals(previous: &OutcomeMetrics, current: &OutcomeMetrics) -> Vec<MetricTrend> {
    ComparisonMetric::ALL
        .into_iter()
        .map(|metric| {
            let previous_value = metric_value(previous, metric);
            let current_value = metric_value(current, metric);
            let delta = current_value
                .zip(previous_value)
                .map(|(current, previous)| current - previous);
            let percent_change = delta.zip(previous_value).and_then(|(delta, previous)| {
                (!previous.is_zero()).then(|| delta / previous.abs() * Decimal::from(100))
            });
            let direction = delta.map(|delta| {
                if delta.is_zero() {
                    TrendDirection::Unchanged
                } else if delta.is_sign_positive() {
                    TrendDirection::Higher
                } else {
                    TrendDirection::Lower
                }
            });

            MetricTrend {
                metric,
                previous: previous_value,
                current: current_value,
                delta,
                percent_change,
                direction,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(yield_kg: i64, revenue: i64, cost: i64) -> OutcomeMetrics {
        let yield_kg = Decimal::from(yield_kg);
        let revenue = Decimal::from(revenue);
        let cost = Decimal::from(cost);
        OutcomeMetrics {
            sellable_yield_kg: Some(yield_kg),
            revenue: Some(revenue),
            total_cost: Some(cost),
            profit: Some(revenue - cost),
            average_price_per_kg: (!yield_kg.is_zero()).then(|| revenue / yield_kg),
            cost_per_kg: (!yield_kg.is_zero()).then(|| cost / yield_kg),
        }
    }

    fn snapshot(id: i64, year: Option<i32>, actual: Option<OutcomeMetrics>) -> SeasonSnapshot {
        SeasonSnapshot {
            season_id: id,
            season_year: year,
            forecast: actual.clone(),
            actual,
        }
    }

    #[test]
    fn history_orders_year_then_id_and_uses_the_first_actual_as_baseline() {
        let rows = build_season_history(vec![
            snapshot(30, Some(2571), Some(metrics(18_000, 1_530_000, 990_000))),
            snapshot(20, Some(2569), Some(metrics(20_000, 1_600_000, 900_000))),
            snapshot(10, Some(2568), None),
            snapshot(40, Some(2571), None),
            snapshot(50, None, None),
        ]);

        assert_eq!(
            rows.iter()
                .map(|row| row.season.season_id)
                .collect::<Vec<_>>(),
            vec![10, 20, 30, 40, 50]
        );
        assert!(!rows[0].baseline);
        assert!(rows[1].baseline);
        assert_eq!(rows[2].previous_actual_season_id, Some(20));
        assert_eq!(rows[2].previous_actual_year, Some(2569));
        assert_eq!(rows[2].actual_trends.len(), 6);
        assert!(rows[3].actual_trends.is_empty());
    }

    #[test]
    fn trend_reports_visible_inputs_delta_direction_and_percent() {
        let trends = compare_actuals(
            &metrics(20_000, 1_600_000, 900_000),
            &metrics(18_000, 1_530_000, 990_000),
        );
        let cost = trends
            .iter()
            .find(|trend| trend.metric == ComparisonMetric::TotalCost)
            .expect("all six trends");

        assert_eq!(cost.previous, Some(Decimal::from(900_000)));
        assert_eq!(cost.current, Some(Decimal::from(990_000)));
        assert_eq!(cost.delta, Some(Decimal::from(90_000)));
        assert_eq!(cost.percent_change, Some(Decimal::from(10)));
        assert_eq!(cost.direction, Some(TrendDirection::Higher));
    }

    #[test]
    fn missing_values_and_zero_baselines_never_invent_a_percent() {
        let previous = metrics(0, 0, 0);
        let mut current = metrics(10, 100, 50);
        current.average_price_per_kg = None;
        let trends = compare_actuals(&previous, &current);

        let revenue = trends
            .iter()
            .find(|trend| trend.metric == ComparisonMetric::Revenue)
            .expect("revenue trend");
        assert_eq!(revenue.delta, Some(Decimal::from(100)));
        assert_eq!(revenue.percent_change, None);

        let price = trends
            .iter()
            .find(|trend| trend.metric == ComparisonMetric::AveragePricePerKg)
            .expect("price trend");
        assert_eq!(price.delta, None);
        assert_eq!(price.direction, None);
    }

    #[test]
    fn skipped_years_compare_only_the_two_observed_actual_seasons() {
        let rows = build_season_history(vec![
            snapshot(1, Some(2569), Some(metrics(20_000, 1_600_000, 900_000))),
            snapshot(2, Some(2570), None),
            snapshot(3, Some(2572), Some(metrics(18_000, 1_530_000, 990_000))),
        ]);

        assert_eq!(rows[2].previous_actual_year, Some(2569));
        assert_eq!(rows[2].previous_actual_season_id, Some(1));
    }
}
