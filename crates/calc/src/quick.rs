use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ForecastMode {
    Quick,
    #[default]
    Detailed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct QuickEstimate {
    pub sellable_yield_kg: Option<Decimal>,
    pub average_price_per_kg: Option<Decimal>,
    pub total_cost: Option<Decimal>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum QuickInputField {
    SellableYieldKg,
    AveragePricePerKg,
    TotalCost,
}

impl QuickInputField {
    pub const ALL: [Self; 3] = [
        Self::SellableYieldKg,
        Self::AveragePricePerKg,
        Self::TotalCost,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum QuickInputIssueKind {
    Missing,
    NotPositive,
    CalculationOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuickInputIssue {
    pub field: Option<QuickInputField>,
    pub kind: QuickInputIssueKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuickAnalysis {
    pub input_issues: Vec<QuickInputIssue>,
    pub revenue: Option<Decimal>,
    pub net_profit: Option<Decimal>,
    pub cost_per_kg: Option<Decimal>,
    pub break_even_price_per_kg: Option<Decimal>,
}

impl QuickEstimate {
    pub fn value(&self, field: QuickInputField) -> Option<Decimal> {
        match field {
            QuickInputField::SellableYieldKg => self.sellable_yield_kg,
            QuickInputField::AveragePricePerKg => self.average_price_per_kg,
            QuickInputField::TotalCost => self.total_cost,
        }
    }

    pub fn first_incomplete(&self) -> Option<QuickInputField> {
        QuickInputField::ALL.into_iter().find(|field| {
            self.value(*field)
                .is_none_or(|value| value <= Decimal::ZERO)
        })
    }
}

pub fn analyze_quick(estimate: &QuickEstimate) -> QuickAnalysis {
    let mut input_issues = Vec::new();
    for field in QuickInputField::ALL {
        match estimate.value(field) {
            None => input_issues.push(QuickInputIssue {
                field: Some(field),
                kind: QuickInputIssueKind::Missing,
            }),
            Some(value) if value <= Decimal::ZERO => input_issues.push(QuickInputIssue {
                field: Some(field),
                kind: QuickInputIssueKind::NotPositive,
            }),
            Some(_) => {}
        }
    }

    if !input_issues.is_empty() {
        return QuickAnalysis {
            input_issues,
            revenue: None,
            net_profit: None,
            cost_per_kg: None,
            break_even_price_per_kg: None,
        };
    }

    let yield_kg = estimate
        .sellable_yield_kg
        .expect("validated quick yield is present");
    let price = estimate
        .average_price_per_kg
        .expect("validated quick price is present");
    let total_cost = estimate
        .total_cost
        .expect("validated quick cost is present");

    let Some(revenue) = yield_kg.checked_mul(price) else {
        input_issues.push(QuickInputIssue {
            field: None,
            kind: QuickInputIssueKind::CalculationOverflow,
        });
        return QuickAnalysis {
            input_issues,
            revenue: None,
            net_profit: None,
            cost_per_kg: None,
            break_even_price_per_kg: None,
        };
    };
    let Some(net_profit) = revenue.checked_sub(total_cost) else {
        input_issues.push(QuickInputIssue {
            field: None,
            kind: QuickInputIssueKind::CalculationOverflow,
        });
        return QuickAnalysis {
            input_issues,
            revenue: None,
            net_profit: None,
            cost_per_kg: None,
            break_even_price_per_kg: None,
        };
    };
    let Some(cost_per_kg) = total_cost.checked_div(yield_kg) else {
        input_issues.push(QuickInputIssue {
            field: None,
            kind: QuickInputIssueKind::CalculationOverflow,
        });
        return QuickAnalysis {
            input_issues,
            revenue: None,
            net_profit: None,
            cost_per_kg: None,
            break_even_price_per_kg: None,
        };
    };

    QuickAnalysis {
        input_issues,
        revenue: Some(revenue),
        net_profit: Some(net_profit),
        cost_per_kg: Some(cost_per_kg),
        break_even_price_per_kg: Some(cost_per_kg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_owner_values_produce_the_first_decision_result() {
        let estimate = QuickEstimate {
            sellable_yield_kg: Some(Decimal::from(20_000)),
            average_price_per_kg: Some(Decimal::from(80)),
            total_cost: Some(Decimal::from(900_000)),
        };

        let result = analyze_quick(&estimate);

        assert!(result.input_issues.is_empty());
        assert_eq!(result.revenue, Some(Decimal::from(1_600_000)));
        assert_eq!(result.net_profit, Some(Decimal::from(700_000)));
        assert_eq!(result.cost_per_kg, Some(Decimal::from(45)));
        assert_eq!(result.break_even_price_per_kg, Some(Decimal::from(45)));
    }

    #[test]
    fn missing_zero_and_negative_inputs_never_produce_partial_results() {
        let estimate = QuickEstimate {
            sellable_yield_kg: None,
            average_price_per_kg: Some(Decimal::ZERO),
            total_cost: Some(Decimal::NEGATIVE_ONE),
        };

        let result = analyze_quick(&estimate);

        assert_eq!(result.input_issues.len(), 3);
        assert_eq!(result.revenue, None);
        assert_eq!(result.net_profit, None);
        assert_eq!(result.cost_per_kg, None);
        assert_eq!(
            estimate.first_incomplete(),
            Some(QuickInputField::SellableYieldKg)
        );
    }

    #[test]
    fn arithmetic_overflow_is_an_explicit_unavailable_result() {
        let estimate = QuickEstimate {
            sellable_yield_kg: Some(Decimal::MAX),
            average_price_per_kg: Some(Decimal::from(2)),
            total_cost: Some(Decimal::ONE),
        };

        let result = analyze_quick(&estimate);

        assert_eq!(
            result.input_issues,
            vec![QuickInputIssue {
                field: None,
                kind: QuickInputIssueKind::CalculationOverflow,
            }]
        );
        assert_eq!(result.revenue, None);
    }

    #[test]
    fn first_incomplete_follows_the_three_question_order() {
        let mut estimate = QuickEstimate::default();
        assert_eq!(
            estimate.first_incomplete(),
            Some(QuickInputField::SellableYieldKg)
        );
        estimate.sellable_yield_kg = Some(Decimal::ONE);
        assert_eq!(
            estimate.first_incomplete(),
            Some(QuickInputField::AveragePricePerKg)
        );
        estimate.average_price_per_kg = Some(Decimal::ONE);
        assert_eq!(
            estimate.first_incomplete(),
            Some(QuickInputField::TotalCost)
        );
        estimate.total_cost = Some(Decimal::ONE);
        assert_eq!(estimate.first_incomplete(), None);
    }
}
