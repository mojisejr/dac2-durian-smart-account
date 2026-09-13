use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::targets::KpiTargets;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub name: String,
    pub market: MarketPlan,
    pub production: ProductionPlan,
    /// What the owner has said about the variable section when it has no
    /// rows. Rows always win: see [`Plan::effective_variable_cost_state`].
    #[serde(default)]
    pub variable_cost_state: CostSectionState,
    pub variable_costs: Vec<VariableCostLine>,
    #[serde(default)]
    pub fixed_cost_state: CostSectionState,
    pub fixed_costs: Vec<FixedCostLine>,
    /// Remembered expenses the owner has not classified yet. They enter no
    /// calculation.
    #[serde(default)]
    pub unclassified_expenses: Vec<UnclassifiedExpense>,
    pub health_answers: Vec<HealthAnswer>,
    pub targets: KpiTargets,
}

/// What is known about a cost section that may have no rows.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum CostSectionState {
    /// The owner has not said; an empty list means nothing yet.
    #[default]
    Unknown,
    /// The owner confirmed the section truly has no cost: a known zero.
    ConfirmedNone,
    /// Rows exist. Derived from the rows, never a stored claim on its own.
    EnteredItems,
}

impl Plan {
    /// Rows always mean entered items; without rows a stored `EnteredItems`
    /// is stale and reads as unknown, never as a silent confirmation.
    pub fn effective_variable_cost_state(&self) -> CostSectionState {
        effective_state(self.variable_cost_state, self.variable_costs.is_empty())
    }

    pub fn effective_fixed_cost_state(&self) -> CostSectionState {
        effective_state(self.fixed_cost_state, self.fixed_costs.is_empty())
    }
}

fn effective_state(stored: CostSectionState, empty: bool) -> CostSectionState {
    match (stored, empty) {
        (_, false) => CostSectionState::EnteredItems,
        (CostSectionState::ConfirmedNone, true) => CostSectionState::ConfirmedNone,
        (_, true) => CostSectionState::Unknown,
    }
}

/// An expense the owner remembers but has not yet said whether it grows with
/// production, is paid in cash this year, or is a multi-year purchase.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UnclassifiedExpense {
    pub name: String,
    pub amount: Option<Decimal>,
    pub note: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketPlan {
    pub target_customer: Option<String>,
    /// The quantity a buyer said they would take this season. It feeds only
    /// the market gap and fulfillment comparison, never the main result.
    pub buyer_committed_kg: Option<Decimal>,
    pub minimum_price_per_kg: Option<Decimal>,
    pub sales_period: Option<String>,
    pub sales_channels: Option<u32>,
    pub largest_buyer_share: Option<Decimal>,
    pub quality_requirements: Option<String>,
}

/// Which production branch feeds sellable kilograms. The unselected branch's
/// facts stay stored so the owner can switch back without re-entering them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum YieldSource {
    /// The owner enters sellable kilograms directly.
    Direct,
    /// Sellable kilograms derive from trees, fruit, weight, and loss.
    #[default]
    Derived,
}

/// Which price branch feeds the revenue price. Grades are kept either way.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PriceSource {
    /// One average price for every kilogram sold.
    Average,
    /// A share and price per grade, weighted into one price.
    #[default]
    ByGrade,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProductionPlan {
    pub yield_source: YieldSource,
    /// Sellable kilograms entered directly; used only when `yield_source` is
    /// `Direct`.
    pub sellable_yield_kg: Option<Decimal>,
    pub area_rai: Option<Decimal>,
    pub producing_trees: Option<Decimal>,
    pub fruits_per_tree: Option<Decimal>,
    pub average_fruit_weight_kg: Option<Decimal>,
    pub loss_share: Option<Decimal>,
    pub price_source: PriceSource,
    /// One average price; used only when `price_source` is `Average`.
    pub average_price_per_kg: Option<Decimal>,
    pub grades: Vec<Grade>,
}

impl ProductionPlan {
    /// Gross kilograms from the orchard facts, before loss. Available whenever
    /// the derived facts exist, whichever branch is selected, so the derived
    /// figure can be shown beside a direct entry.
    pub fn derived_gross_yield_kg(&self) -> Option<Decimal> {
        Some(self.producing_trees? * self.fruits_per_tree? * self.average_fruit_weight_kg?)
    }

    /// Sellable kilograms from the orchard facts after loss.
    pub fn derived_sellable_yield_kg(&self) -> Option<Decimal> {
        Some(self.derived_gross_yield_kg()? * (Decimal::ONE - self.loss_share?))
    }

    /// Sellable kilograms from the selected branch only.
    pub fn selected_sellable_yield_kg(&self) -> Option<Decimal> {
        match self.yield_source {
            YieldSource::Direct => self.sellable_yield_kg,
            YieldSource::Derived => self.derived_sellable_yield_kg(),
        }
    }

    /// The by-grade weighted price, available whenever every grade carries a
    /// share and a price, whichever branch is selected.
    pub fn weighted_grade_price_per_kg(&self) -> Option<Decimal> {
        if self.grades.is_empty() {
            return None;
        }
        self.grades.iter().try_fold(Decimal::ZERO, |total, grade| {
            Some(total + grade.share? * grade.price_per_kg?)
        })
    }

    /// The price per kilogram from the selected branch only.
    pub fn selected_price_per_kg(&self) -> Option<Decimal> {
        match self.price_source {
            PriceSource::Average => self.average_price_per_kg,
            PriceSource::ByGrade => self.weighted_grade_price_per_kg(),
        }
    }
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
    /// A known total for the line instead of quantity times unit price. A
    /// line carrying both is an input issue, never resolved silently.
    #[serde(default)]
    pub total_amount: Option<Decimal>,
}

impl VariableCostLine {
    /// Whether this line is entered as one total rather than per unit.
    pub fn is_total_only(&self) -> bool {
        self.total_amount.is_some()
    }

    /// The quantity that feeds the line total and per-unit figures. A
    /// total-only line has no quantity, so it yields no per-unit figure.
    pub fn effective_quantity(&self, sellable_yield_kg: Option<Decimal>) -> Option<Decimal> {
        if self.is_total_only() {
            None
        } else if self.kind.follows_sellable_yield() {
            self.quantity.or(sellable_yield_kg)
        } else {
            self.quantity
        }
    }

    /// Whether the sellable-yield default is what fills this line's quantity.
    pub fn uses_sellable_yield_default(&self) -> bool {
        !self.is_total_only() && self.kind.follows_sellable_yield() && self.quantity.is_none()
    }

    pub fn total(&self, sellable_yield_kg: Option<Decimal>) -> Option<Decimal> {
        if let Some(total) = self.total_amount {
            return Some(total);
        }
        Some(self.effective_quantity(sellable_yield_kg)? * self.unit_price?)
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
