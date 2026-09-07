use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KpiKind {
    YieldPerRai,
    YieldPerTree,
    YieldPerLaborDay,
    YieldPerFertilizerKg,
    YieldPerWaterCubicMeter,
    YieldPerKwh,
    QualityGradeShare,
    LossShare,
    CostPerKg,
}

impl KpiKind {
    pub const ALL: [Self; 9] = [
        Self::YieldPerRai,
        Self::YieldPerTree,
        Self::YieldPerLaborDay,
        Self::YieldPerFertilizerKg,
        Self::YieldPerWaterCubicMeter,
        Self::YieldPerKwh,
        Self::QualityGradeShare,
        Self::LossShare,
        Self::CostPerKg,
    ];

    pub const fn lower_is_better(self) -> bool {
        matches!(self, Self::LossShare | Self::CostPerKg)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KpiTargets {
    pub yield_per_rai: Option<Decimal>,
    pub yield_per_tree: Option<Decimal>,
    pub yield_per_labor_day: Option<Decimal>,
    pub yield_per_fertilizer_kg: Option<Decimal>,
    pub yield_per_water_cubic_meter: Option<Decimal>,
    pub yield_per_kwh: Option<Decimal>,
    pub quality_grade_share: Option<Decimal>,
    pub loss_share: Option<Decimal>,
    pub cost_per_kg: Option<Decimal>,
}

impl KpiTargets {
    pub const fn get(&self, kind: KpiKind) -> Option<Decimal> {
        match kind {
            KpiKind::YieldPerRai => self.yield_per_rai,
            KpiKind::YieldPerTree => self.yield_per_tree,
            KpiKind::YieldPerLaborDay => self.yield_per_labor_day,
            KpiKind::YieldPerFertilizerKg => self.yield_per_fertilizer_kg,
            KpiKind::YieldPerWaterCubicMeter => self.yield_per_water_cubic_meter,
            KpiKind::YieldPerKwh => self.yield_per_kwh,
            KpiKind::QualityGradeShare => self.quality_grade_share,
            KpiKind::LossShare => self.loss_share,
            KpiKind::CostPerKg => self.cost_per_kg,
        }
    }

    pub fn values(&self) -> impl Iterator<Item = (KpiKind, Option<Decimal>)> + '_ {
        KpiKind::ALL.into_iter().map(|kind| (kind, self.get(kind)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KpiVerdict {
    Met,
    Improve,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unset_target_is_distinct_from_zero() {
        let targets = KpiTargets::default();
        assert_eq!(targets.get(KpiKind::YieldPerRai), None);

        let targets = KpiTargets {
            yield_per_rai: Some(Decimal::ZERO),
            ..KpiTargets::default()
        };
        assert_eq!(targets.get(KpiKind::YieldPerRai), Some(Decimal::ZERO));
    }
}
