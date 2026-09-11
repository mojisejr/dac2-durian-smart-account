use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const MAX_ASSET_VALUE: i64 = 1_000_000_000_000;
pub const MAX_USEFUL_LIFE_YEARS: u32 = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AssetKind {
    Equipment,
    OwnedLand,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetFacts {
    pub name: String,
    pub kind: AssetKind,
    pub original_cost: Decimal,
    pub start_year: i32,
    pub useful_life_years: Option<u32>,
    pub residual_value: Option<Decimal>,
    pub retired_year: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetAllocation {
    pub facts: AssetFacts,
    pub season_year: i32,
    pub annual_depreciation: Decimal,
    pub investment_value: Decimal,
    pub residual_assumed_zero: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetRuleError {
    EmptyName,
    OriginalCostOutOfRange,
    StartYearOutOfRange,
    UsefulLifeRequired,
    UsefulLifeOutOfRange,
    ResidualValueOutOfRange,
    RetiredYearOutOfRange,
    YearOverflow,
}

pub fn start_year_from_prior_use(
    season_year: i32,
    approximate_years_in_use: u32,
) -> Result<i32, AssetRuleError> {
    valid_year(season_year)
        .then_some(())
        .ok_or(AssetRuleError::StartYearOutOfRange)?;
    if approximate_years_in_use > MAX_USEFUL_LIFE_YEARS {
        return Err(AssetRuleError::UsefulLifeOutOfRange);
    }
    let years =
        i32::try_from(approximate_years_in_use).map_err(|_| AssetRuleError::YearOverflow)?;
    let start_year = season_year
        .checked_sub(years)
        .ok_or(AssetRuleError::YearOverflow)?;
    valid_year(start_year)
        .then_some(start_year)
        .ok_or(AssetRuleError::StartYearOutOfRange)
}

pub fn allocate(
    facts: &AssetFacts,
    season_year: i32,
) -> Result<Option<AssetAllocation>, AssetRuleError> {
    validate(facts)?;
    if !valid_year(season_year) {
        return Err(AssetRuleError::StartYearOutOfRange);
    }
    if season_year < facts.start_year
        || facts
            .retired_year
            .is_some_and(|retired| season_year >= retired)
    {
        return Ok(None);
    }

    let (annual_depreciation, residual_assumed_zero) = match facts.kind {
        AssetKind::OwnedLand => (Decimal::ZERO, false),
        AssetKind::Equipment => {
            let life = facts.useful_life_years.expect("validated equipment life");
            let end_year = facts
                .start_year
                .checked_add(i32::try_from(life).map_err(|_| AssetRuleError::YearOverflow)?)
                .ok_or(AssetRuleError::YearOverflow)?;
            if season_year >= end_year {
                return Ok(None);
            }
            let residual = facts.residual_value.unwrap_or(Decimal::ZERO);
            (
                (facts.original_cost - residual) / Decimal::from(life),
                facts.residual_value.is_none(),
            )
        }
    };

    Ok(Some(AssetAllocation {
        facts: facts.clone(),
        season_year,
        annual_depreciation,
        investment_value: facts.original_cost,
        residual_assumed_zero,
    }))
}

pub fn validate(facts: &AssetFacts) -> Result<(), AssetRuleError> {
    if facts.name.trim().is_empty() {
        return Err(AssetRuleError::EmptyName);
    }
    if facts.original_cost <= Decimal::ZERO || facts.original_cost > Decimal::from(MAX_ASSET_VALUE)
    {
        return Err(AssetRuleError::OriginalCostOutOfRange);
    }
    if !valid_year(facts.start_year) {
        return Err(AssetRuleError::StartYearOutOfRange);
    }
    if let Some(retired_year) = facts.retired_year
        && (!valid_year(retired_year) || retired_year < facts.start_year)
    {
        return Err(AssetRuleError::RetiredYearOutOfRange);
    }
    match facts.kind {
        AssetKind::OwnedLand => {
            if facts.useful_life_years.is_some() || facts.residual_value.is_some() {
                return Err(AssetRuleError::UsefulLifeOutOfRange);
            }
        }
        AssetKind::Equipment => {
            let Some(life) = facts.useful_life_years else {
                return Err(AssetRuleError::UsefulLifeRequired);
            };
            if life == 0 || life > MAX_USEFUL_LIFE_YEARS {
                return Err(AssetRuleError::UsefulLifeOutOfRange);
            }
            if let Some(residual) = facts.residual_value
                && (residual < Decimal::ZERO || residual > facts.original_cost)
            {
                return Err(AssetRuleError::ResidualValueOutOfRange);
            }
            facts
                .start_year
                .checked_add(i32::try_from(life).map_err(|_| AssetRuleError::YearOverflow)?)
                .ok_or(AssetRuleError::YearOverflow)?;
        }
    }
    Ok(())
}

const fn valid_year(year: i32) -> bool {
    year >= 1000 && year <= 9999
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equipment() -> AssetFacts {
        AssetFacts {
            name: "ระบบน้ำ".into(),
            kind: AssetKind::Equipment,
            original_cost: Decimal::from(100_000),
            start_year: 2568,
            useful_life_years: Some(5),
            residual_value: None,
            retired_year: None,
        }
    }

    #[test]
    fn equipment_is_active_from_start_until_the_exclusive_end_year() {
        assert!(allocate(&equipment(), 2567).unwrap().is_none());
        let first = allocate(&equipment(), 2568).unwrap().unwrap();
        assert_eq!(first.annual_depreciation, Decimal::from(20_000));
        assert!(first.residual_assumed_zero);
        assert!(allocate(&equipment(), 2572).unwrap().is_some());
        assert!(allocate(&equipment(), 2573).unwrap().is_none());
    }

    #[test]
    fn approximate_prior_use_converts_against_an_explicit_season_not_the_clock() {
        assert_eq!(start_year_from_prior_use(2570, 3), Ok(2567));
        assert_eq!(
            start_year_from_prior_use(2570, 201),
            Err(AssetRuleError::UsefulLifeOutOfRange)
        );
    }

    #[test]
    fn retirement_is_exclusive_and_owned_land_never_depreciates() {
        let land = AssetFacts {
            name: "ที่ดินสวน".into(),
            kind: AssetKind::OwnedLand,
            original_cost: Decimal::from(2_000_000),
            start_year: 2550,
            useful_life_years: None,
            residual_value: None,
            retired_year: Some(2570),
        };
        let active = allocate(&land, 2569).unwrap().unwrap();
        assert_eq!(active.annual_depreciation, Decimal::ZERO);
        assert_eq!(active.investment_value, Decimal::from(2_000_000));
        assert!(allocate(&land, 2570).unwrap().is_none());
    }

    #[test]
    fn residual_value_changes_depreciation_but_cannot_exceed_cost() {
        let mut asset = equipment();
        asset.residual_value = Some(Decimal::from(25_000));
        assert_eq!(
            allocate(&asset, 2568).unwrap().unwrap().annual_depreciation,
            Decimal::from(15_000)
        );
        asset.residual_value = Some(asset.original_cost);
        assert_eq!(
            allocate(&asset, 2568).unwrap().unwrap().annual_depreciation,
            Decimal::ZERO
        );
        asset.residual_value = Some(asset.original_cost + Decimal::ONE);
        assert_eq!(
            validate(&asset),
            Err(AssetRuleError::ResidualValueOutOfRange)
        );
    }

    #[test]
    fn zero_negative_and_unreasonably_large_values_are_rejected() {
        let mut asset = equipment();
        asset.original_cost = Decimal::ZERO;
        assert_eq!(
            validate(&asset),
            Err(AssetRuleError::OriginalCostOutOfRange)
        );
        asset.original_cost = Decimal::NEGATIVE_ONE;
        assert_eq!(
            validate(&asset),
            Err(AssetRuleError::OriginalCostOutOfRange)
        );
        asset.original_cost = Decimal::from(MAX_ASSET_VALUE) + Decimal::ONE;
        assert_eq!(
            validate(&asset),
            Err(AssetRuleError::OriginalCostOutOfRange)
        );
        asset.original_cost = Decimal::from(100_000);
        asset.useful_life_years = Some(0);
        assert_eq!(validate(&asset), Err(AssetRuleError::UsefulLifeOutOfRange));
        asset.useful_life_years = Some(MAX_USEFUL_LIFE_YEARS + 1);
        assert_eq!(validate(&asset), Err(AssetRuleError::UsefulLifeOutOfRange));
    }
}
