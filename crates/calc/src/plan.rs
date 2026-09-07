use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::targets::KpiTargets;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub name: String,
    pub market: MarketPlan,
    pub production: ProductionPlan,
    pub variable_costs: Vec<VariableCostLine>,
    pub fixed_costs: Vec<FixedCostLine>,
    pub health_answers: Vec<HealthAnswer>,
    pub targets: KpiTargets,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketPlan {
    pub target_customer: Option<String>,
    pub demand_kg: Option<Decimal>,
    pub minimum_price_per_kg: Option<Decimal>,
    pub sales_period: Option<String>,
    pub sales_channels: Option<u32>,
    pub largest_buyer_share: Option<Decimal>,
    pub quality_requirements: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProductionPlan {
    pub area_rai: Option<Decimal>,
    pub producing_trees: Option<Decimal>,
    pub fruits_per_tree: Option<Decimal>,
    pub average_fruit_weight_kg: Option<Decimal>,
    pub loss_share: Option<Decimal>,
    pub grades: Vec<Grade>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grade {
    pub name: String,
    pub share: Option<Decimal>,
    pub price_per_kg: Option<Decimal>,
    /// Whether this owner-defined grade contributes to the workbook's
    /// quality-grade KPI (the sample workbook calls it "A+B").
    pub counts_as_quality_grade: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VariableCostKind {
    Fertilizer,
    CropProtection,
    Water,
    OrchardLabor,
    Electricity,
    Fuel,
    HarvestLabor,
    Transport,
    Packing,
    Maintenance,
    Other,
}

impl VariableCostKind {
    pub const fn follows_sellable_yield(self) -> bool {
        matches!(self, Self::HarvestLabor | Self::Transport | Self::Packing)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableCostLine {
    pub name: String,
    pub kind: VariableCostKind,
    /// `None` on harvest, transport, and packing means use sellable yield.
    /// Supplying a value is an explicit orchard-specific override.
    pub quantity: Option<Decimal>,
    pub unit: String,
    pub unit_price: Option<Decimal>,
}

impl VariableCostLine {
    pub fn effective_quantity(&self, sellable_yield_kg: Option<Decimal>) -> Option<Decimal> {
        if self.kind.follows_sellable_yield() {
            self.quantity.or(sellable_yield_kg)
        } else {
            self.quantity
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CashKind {
    Cash,
    NonCash,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixedCostLine {
    pub name: String,
    pub cash_kind: CashKind,
    pub amount_per_year: Option<Decimal>,
    pub investment_base: Option<Decimal>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum HealthDimension {
    Finance,
    Production,
    Market,
    Resources,
    People,
    Resilience,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum HealthQuestion {
    ProfitAndCash,
    NextSeasonReserve,
    YieldAndQuality,
    LossControl,
    MultipleSalesChannels,
    PriceVolatility,
    ResourceEfficiency,
    EnvironmentalCare,
    FairAndSafeWork,
    LaborContinuity,
    DownsideSurvival,
    ContingencyPlan,
}

impl HealthQuestion {
    pub const ALL: [Self; 12] = [
        Self::ProfitAndCash,
        Self::NextSeasonReserve,
        Self::YieldAndQuality,
        Self::LossControl,
        Self::MultipleSalesChannels,
        Self::PriceVolatility,
        Self::ResourceEfficiency,
        Self::EnvironmentalCare,
        Self::FairAndSafeWork,
        Self::LaborContinuity,
        Self::DownsideSurvival,
        Self::ContingencyPlan,
    ];

    pub const fn dimension(self) -> HealthDimension {
        match self {
            Self::ProfitAndCash | Self::NextSeasonReserve => HealthDimension::Finance,
            Self::YieldAndQuality | Self::LossControl => HealthDimension::Production,
            Self::MultipleSalesChannels | Self::PriceVolatility => HealthDimension::Market,
            Self::ResourceEfficiency | Self::EnvironmentalCare => HealthDimension::Resources,
            Self::FairAndSafeWork | Self::LaborContinuity => HealthDimension::People,
            Self::DownsideSurvival | Self::ContingencyPlan => HealthDimension::Resilience,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthAnswer {
    pub question: HealthQuestion,
    pub score: Option<u8>,
}
