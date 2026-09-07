use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::plan::{HealthDimension, VariableCostKind};
use crate::targets::{KpiKind, KpiVerdict};
use crate::validation::InputIssue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub input_issues: Vec<InputIssue>,
    pub revenue: RevenueAnalysis,
    pub cost: CostAnalysis,
    pub business: BusinessAnalysis,
    pub efficiency: Vec<KpiResult>,
    pub tax: TaxAnalysis,
    pub scenario: ScenarioAnalysis,
    pub checks: CompletenessAnalysis,
    pub health: HealthAnalysis,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RevenueAnalysis {
    pub gross_yield_kg: Option<Decimal>,
    pub sellable_yield_kg: Option<Decimal>,
    pub market_gap_kg: Option<Decimal>,
    pub grade_share_total: Option<Decimal>,
    pub weighted_price_per_kg: Option<Decimal>,
    pub revenue: Option<Decimal>,
    pub market_fulfillment: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableCostResult {
    pub kind: VariableCostKind,
    pub quantity: Option<Decimal>,
    pub total: Option<Decimal>,
    pub yield_per_unit: Option<Decimal>,
    pub share_of_variable_cost: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub variable_lines: Vec<VariableCostResult>,
    pub area_rai: Option<Decimal>,
    pub variable_cost: Option<Decimal>,
    pub variable_cost_per_kg: Option<Decimal>,
    pub fixed_cost: Option<Decimal>,
    pub cash_fixed_cost: Option<Decimal>,
    pub investment_base: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub cost_per_kg: Option<Decimal>,
    pub cost_per_rai: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BusinessAnalysis {
    pub net_profit: Option<Decimal>,
    pub net_margin: Option<Decimal>,
    pub contribution_per_kg: Option<Decimal>,
    pub break_even_kg: Option<Decimal>,
    pub break_even_kg_per_rai: Option<Decimal>,
    pub break_even_revenue: Option<Decimal>,
    pub safety_margin_kg: Option<Decimal>,
    pub roi: Option<Decimal>,
    pub operating_cash_flow: Option<Decimal>,
    pub payback_years: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KpiResult {
    pub kind: KpiKind,
    pub actual: Option<Decimal>,
    pub target: Option<Decimal>,
    pub verdict: Option<KpiVerdict>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaxMethodAnalysis {
    pub income: Option<Decimal>,
    pub expense: Option<Decimal>,
    pub personal_allowance: Decimal,
    pub taxable_income: Option<Decimal>,
    pub estimated_tax: Option<Decimal>,
    pub average_tax_rate: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaxAnalysis {
    pub actual_expense: TaxMethodAnalysis,
    pub flat_sixty_percent: TaxMethodAnalysis,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioAnalysis {
    pub yield_changes: [Decimal; 5],
    pub price_changes: [Decimal; 5],
    pub profits: [[Option<Decimal>; 5]; 5],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CheckKind {
    GradeSharesTotalOne,
    SellableYieldPositive,
    PriceAboveVariableCost,
    InvestmentPositive,
    TotalCostLinked,
    HealthAnswersComplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CheckStatus {
    Ok,
    Warning,
    NeedsCheck,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    pub kind: CheckKind,
    pub actual: Option<Decimal>,
    pub expected: Option<Decimal>,
    pub difference: Option<Decimal>,
    pub status: CheckStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Readiness {
    Ready,
    NeedsReview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompletenessAnalysis {
    pub checks: Vec<CheckResult>,
    pub overall: Readiness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Strong,
    Watch,
    ImproveUrgently,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthDimensionResult {
    pub dimension: HealthDimension,
    pub average: Option<Decimal>,
    pub status: Option<HealthStatus>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthAnalysis {
    pub dimensions: Vec<HealthDimensionResult>,
    pub overall_score: Option<Decimal>,
    pub overall_status: Option<HealthStatus>,
}
