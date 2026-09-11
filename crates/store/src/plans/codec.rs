use calc::{CashKind, ForecastMode, HealthQuestion, VariableCostKind};

use super::StoreError;

pub fn forecast_mode(value: ForecastMode) -> &'static str {
    match value {
        ForecastMode::Quick => "quick",
        ForecastMode::Detailed => "detailed",
    }
}

pub fn parse_forecast_mode(value: String) -> Result<ForecastMode, StoreError> {
    match value.as_str() {
        "quick" => Ok(ForecastMode::Quick),
        "detailed" => Ok(ForecastMode::Detailed),
        _ => Err(StoreError::InvalidValue {
            field: "plans.forecast_mode",
            value,
        }),
    }
}

pub fn variable_cost_kind(value: VariableCostKind) -> &'static str {
    match value {
        VariableCostKind::Fertilizer => "fertilizer",
        VariableCostKind::CropProtection => "crop_protection",
        VariableCostKind::Water => "water",
        VariableCostKind::OrchardLabor => "orchard_labor",
        VariableCostKind::Electricity => "electricity",
        VariableCostKind::Fuel => "fuel",
        VariableCostKind::HarvestLabor => "harvest_labor",
        VariableCostKind::Transport => "transport",
        VariableCostKind::Packing => "packing",
        VariableCostKind::Maintenance => "maintenance",
        VariableCostKind::Other => "other",
    }
}

pub fn parse_variable_cost_kind(value: String) -> Result<VariableCostKind, StoreError> {
    match value.as_str() {
        "fertilizer" => Ok(VariableCostKind::Fertilizer),
        "crop_protection" => Ok(VariableCostKind::CropProtection),
        "water" => Ok(VariableCostKind::Water),
        "orchard_labor" => Ok(VariableCostKind::OrchardLabor),
        "electricity" => Ok(VariableCostKind::Electricity),
        "fuel" => Ok(VariableCostKind::Fuel),
        "harvest_labor" => Ok(VariableCostKind::HarvestLabor),
        "transport" => Ok(VariableCostKind::Transport),
        "packing" => Ok(VariableCostKind::Packing),
        "maintenance" => Ok(VariableCostKind::Maintenance),
        "other" => Ok(VariableCostKind::Other),
        _ => Err(StoreError::InvalidValue {
            field: "variable_cost_lines.kind",
            value,
        }),
    }
}

pub fn cash_kind(value: CashKind) -> &'static str {
    match value {
        CashKind::Cash => "cash",
        CashKind::NonCash => "non_cash",
    }
}

pub fn parse_cash_kind(value: String) -> Result<CashKind, StoreError> {
    match value.as_str() {
        "cash" => Ok(CashKind::Cash),
        "non_cash" => Ok(CashKind::NonCash),
        _ => Err(StoreError::InvalidValue {
            field: "fixed_cost_lines.cash_kind",
            value,
        }),
    }
}

pub fn health_question(value: HealthQuestion) -> &'static str {
    match value {
        HealthQuestion::ProfitAndCash => "profit_and_cash",
        HealthQuestion::NextSeasonReserve => "next_season_reserve",
        HealthQuestion::YieldAndQuality => "yield_and_quality",
        HealthQuestion::LossControl => "loss_control",
        HealthQuestion::MultipleSalesChannels => "multiple_sales_channels",
        HealthQuestion::PriceVolatility => "price_volatility",
        HealthQuestion::ResourceEfficiency => "resource_efficiency",
        HealthQuestion::EnvironmentalCare => "environmental_care",
        HealthQuestion::FairAndSafeWork => "fair_and_safe_work",
        HealthQuestion::LaborContinuity => "labor_continuity",
        HealthQuestion::DownsideSurvival => "downside_survival",
        HealthQuestion::ContingencyPlan => "contingency_plan",
    }
}

pub fn parse_health_question(value: String) -> Result<HealthQuestion, StoreError> {
    match value.as_str() {
        "profit_and_cash" => Ok(HealthQuestion::ProfitAndCash),
        "next_season_reserve" => Ok(HealthQuestion::NextSeasonReserve),
        "yield_and_quality" => Ok(HealthQuestion::YieldAndQuality),
        "loss_control" => Ok(HealthQuestion::LossControl),
        "multiple_sales_channels" => Ok(HealthQuestion::MultipleSalesChannels),
        "price_volatility" => Ok(HealthQuestion::PriceVolatility),
        "resource_efficiency" => Ok(HealthQuestion::ResourceEfficiency),
        "environmental_care" => Ok(HealthQuestion::EnvironmentalCare),
        "fair_and_safe_work" => Ok(HealthQuestion::FairAndSafeWork),
        "labor_continuity" => Ok(HealthQuestion::LaborContinuity),
        "downside_survival" => Ok(HealthQuestion::DownsideSurvival),
        "contingency_plan" => Ok(HealthQuestion::ContingencyPlan),
        _ => Err(StoreError::InvalidValue {
            field: "health_answers.question",
            value,
        }),
    }
}
