use rust_decimal::Decimal;

use crate::cost::quantity_for_kind;
use crate::{
    CostAnalysis, KpiKind, KpiResult, KpiVerdict, Plan, RevenueAnalysis, VariableCostKind,
};

pub fn calculate(plan: &Plan, revenue: &RevenueAnalysis, cost: &CostAnalysis) -> Vec<KpiResult> {
    KpiKind::ALL
        .into_iter()
        .map(|kind| {
            let actual = actual(kind, plan, revenue, cost);
            let target = plan.targets.get(kind);
            KpiResult {
                kind,
                actual,
                target,
                verdict: actual
                    .zip(target)
                    .map(|(actual, target)| verdict(kind, actual, target)),
            }
        })
        .collect()
}

fn actual(
    kind: KpiKind,
    plan: &Plan,
    revenue: &RevenueAnalysis,
    cost: &CostAnalysis,
) -> Option<Decimal> {
    match kind {
        KpiKind::YieldPerRai => ratio(revenue.sellable_yield_kg, plan.production.area_rai),
        KpiKind::YieldPerTree => ratio(revenue.sellable_yield_kg, plan.production.producing_trees),
        KpiKind::YieldPerLaborDay => ratio(
            revenue.sellable_yield_kg,
            quantity_for_kind(plan, revenue, VariableCostKind::OrchardLabor),
        ),
        KpiKind::YieldPerFertilizerKg => ratio(
            revenue.sellable_yield_kg,
            quantity_for_kind(plan, revenue, VariableCostKind::Fertilizer),
        ),
        KpiKind::YieldPerWaterCubicMeter => ratio(
            revenue.sellable_yield_kg,
            quantity_for_kind(plan, revenue, VariableCostKind::Water),
        ),
        KpiKind::YieldPerKwh => ratio(
            revenue.sellable_yield_kg,
            quantity_for_kind(plan, revenue, VariableCostKind::Electricity),
        ),
        KpiKind::QualityGradeShare => quality_grade_share(plan),
        KpiKind::LossShare => plan.production.loss_share,
        KpiKind::CostPerKg => cost.cost_per_kg,
    }
}

fn ratio(numerator: Option<Decimal>, denominator: Option<Decimal>) -> Option<Decimal> {
    numerator
        .zip(denominator)
        .and_then(|(numerator, denominator)| {
            (!denominator.is_zero()).then(|| numerator / denominator)
        })
}

fn quality_grade_share(plan: &Plan) -> Option<Decimal> {
    if plan.production.grades.is_empty() {
        return None;
    }
    plan.production
        .grades
        .iter()
        .try_fold(Decimal::ZERO, |total, grade| {
            grade.share.map(|share| {
                total
                    + if grade.counts_as_quality_grade {
                        share
                    } else {
                        Decimal::ZERO
                    }
            })
        })
}

fn verdict(kind: KpiKind, actual: Decimal, target: Decimal) -> KpiVerdict {
    let met = if kind.lower_is_better() {
        actual <= target
    } else {
        actual >= target
    };
    if met {
        KpiVerdict::Met
    } else {
        KpiVerdict::Improve
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{VariableCostLine, cost, revenue, workbook_sample};

    fn results(plan: &Plan) -> Vec<KpiResult> {
        let revenue = revenue::calculate(plan);
        let cost = cost::calculate(plan, &revenue);
        calculate(plan, &revenue, &cost)
    }

    fn find(results: &[KpiResult], kind: KpiKind) -> &KpiResult {
        results
            .iter()
            .find(|result| result.kind == kind)
            .expect("all nine KPI results")
    }

    #[test]
    fn workbook_kpis_and_directions_match() {
        let results = results(&workbook_sample());

        assert_eq!(results.len(), 9);
        assert_eq!(
            find(&results, KpiKind::YieldPerRai).actual,
            Some(Decimal::from(1_995))
        );
        assert_eq!(
            find(&results, KpiKind::YieldPerLaborDay).actual,
            Some(Decimal::new(1_995, 1))
        );
        assert_eq!(
            find(&results, KpiKind::QualityGradeShare).actual,
            Some(Decimal::new(8, 1))
        );
        assert_eq!(
            find(&results, KpiKind::YieldPerLaborDay).verdict,
            Some(KpiVerdict::Met)
        );
        assert_eq!(
            find(&results, KpiKind::LossShare).verdict,
            Some(KpiVerdict::Met)
        );
        assert_eq!(
            find(&results, KpiKind::YieldPerRai).verdict,
            Some(KpiVerdict::Improve)
        );
    }

    #[test]
    fn unset_exact_better_and_worse_targets_have_distinct_verdicts() {
        let mut plan = workbook_sample();
        plan.targets.yield_per_rai = None;
        assert_eq!(find(&results(&plan), KpiKind::YieldPerRai).verdict, None);

        plan.targets.yield_per_rai = Some(Decimal::from(1_995));
        assert_eq!(
            find(&results(&plan), KpiKind::YieldPerRai).verdict,
            Some(KpiVerdict::Met)
        );
        plan.targets.yield_per_rai = Some(Decimal::from(1_900));
        assert_eq!(
            find(&results(&plan), KpiKind::YieldPerRai).verdict,
            Some(KpiVerdict::Met)
        );
        plan.targets.yield_per_rai = Some(Decimal::from(2_000));
        assert_eq!(
            find(&results(&plan), KpiKind::YieldPerRai).verdict,
            Some(KpiVerdict::Improve)
        );

        plan.targets.loss_share = Some(Decimal::new(5, 2));
        assert_eq!(
            find(&results(&plan), KpiKind::LossShare).verdict,
            Some(KpiVerdict::Met)
        );
        plan.targets.loss_share = Some(Decimal::new(4, 2));
        assert_eq!(
            find(&results(&plan), KpiKind::LossShare).verdict,
            Some(KpiVerdict::Improve)
        );
    }

    #[test]
    fn other_cost_rows_never_feed_a_kind_specific_kpi() {
        let mut plan = workbook_sample();
        let before = find(&results(&plan), KpiKind::YieldPerFertilizerKg).actual;
        plan.variable_costs.push(VariableCostLine {
            name: "ปุ๋ยที่ตั้งชื่อเองแต่เป็นอื่นๆ".into(),
            kind: VariableCostKind::Other,
            quantity: Some(Decimal::from(999_999)),
            unit: "กก.".into(),
            unit_price: Some(Decimal::ONE),
        });
        assert_eq!(
            find(&results(&plan), KpiKind::YieldPerFertilizerKg).actual,
            before
        );
    }

    #[test]
    fn a_zero_kpi_denominator_is_unavailable() {
        let mut plan = workbook_sample();
        plan.production.area_rai = Some(Decimal::ZERO);
        assert_eq!(find(&results(&plan), KpiKind::YieldPerRai).actual, None);
    }

    #[test]
    fn every_input_efficiency_denominator_handles_zero_and_absence() {
        let cases = [
            (VariableCostKind::OrchardLabor, KpiKind::YieldPerLaborDay),
            (VariableCostKind::Fertilizer, KpiKind::YieldPerFertilizerKg),
            (VariableCostKind::Water, KpiKind::YieldPerWaterCubicMeter),
            (VariableCostKind::Electricity, KpiKind::YieldPerKwh),
        ];
        for (cost_kind, kpi_kind) in cases {
            let mut plan = workbook_sample();
            let line = plan
                .variable_costs
                .iter()
                .position(|line| line.kind == cost_kind)
                .expect("sample has each KPI cost kind");
            plan.variable_costs[line].quantity = Some(Decimal::ZERO);
            assert_eq!(find(&results(&plan), kpi_kind).actual, None);

            plan.variable_costs[line].quantity = None;
            assert_eq!(find(&results(&plan), kpi_kind).actual, None);
        }

        let mut plan = workbook_sample();
        plan.production.producing_trees = Some(Decimal::ZERO);
        assert_eq!(find(&results(&plan), KpiKind::YieldPerTree).actual, None);
        plan.production.producing_trees = None;
        assert_eq!(find(&results(&plan), KpiKind::YieldPerTree).actual, None);
    }
}
