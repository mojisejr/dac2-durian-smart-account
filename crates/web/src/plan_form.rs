use std::str::FromStr;

use calc::{
    CashKind, FixedCostLine, Grade, HealthAnswer, HealthQuestion, KpiTargets, MarketPlan, Plan,
    ProductionPlan, VariableCostKind, VariableCostLine,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanForm {
    pub name: String,
    pub market: MarketForm,
    pub production: ProductionForm,
    pub grades: Vec<GradeForm>,
    pub variable_costs: Vec<VariableCostForm>,
    pub fixed_costs: Vec<FixedCostForm>,
    pub targets: TargetsForm,
    pub health_scores: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketForm {
    pub target_customer: String,
    pub demand_kg: String,
    pub minimum_price_per_kg: String,
    pub sales_period: String,
    pub sales_channels: String,
    pub largest_buyer_percent: String,
    pub quality_requirements: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProductionForm {
    pub area_rai: String,
    pub producing_trees: String,
    pub fruits_per_tree: String,
    pub average_fruit_weight_kg: String,
    pub loss_percent: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GradeForm {
    pub name: String,
    pub share_percent: String,
    pub price_per_kg: String,
    pub counts_as_quality_grade: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableCostForm {
    pub name: String,
    pub kind: VariableCostKind,
    pub quantity: String,
    pub unit: String,
    pub unit_price: String,
}

impl Default for VariableCostForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: VariableCostKind::Other,
            quantity: String::new(),
            unit: String::new(),
            unit_price: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixedCostForm {
    pub name: String,
    pub cash_kind: CashKind,
    pub amount_per_year: String,
    pub investment_base: String,
}

impl Default for FixedCostForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            cash_kind: CashKind::Cash,
            amount_per_year: String::new(),
            investment_base: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TargetsForm {
    pub yield_per_rai: String,
    pub yield_per_tree: String,
    pub yield_per_labor_day: String,
    pub yield_per_fertilizer_kg: String,
    pub yield_per_water_cubic_meter: String,
    pub yield_per_kwh: String,
    pub quality_grade_percent: String,
    pub loss_percent: String,
    pub cost_per_kg: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormError {
    pub field: String,
    pub message: String,
}

impl PlanForm {
    pub fn from_plan(plan: &Plan) -> Self {
        Self {
            name: plan.name.clone(),
            market: MarketForm {
                target_customer: text(&plan.market.target_customer),
                demand_kg: decimal(plan.market.demand_kg),
                minimum_price_per_kg: decimal(plan.market.minimum_price_per_kg),
                sales_period: text(&plan.market.sales_period),
                sales_channels: plan
                    .market
                    .sales_channels
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                largest_buyer_percent: percent(plan.market.largest_buyer_share),
                quality_requirements: text(&plan.market.quality_requirements),
            },
            production: ProductionForm {
                area_rai: decimal(plan.production.area_rai),
                producing_trees: decimal(plan.production.producing_trees),
                fruits_per_tree: decimal(plan.production.fruits_per_tree),
                average_fruit_weight_kg: decimal(plan.production.average_fruit_weight_kg),
                loss_percent: percent(plan.production.loss_share),
            },
            grades: plan
                .production
                .grades
                .iter()
                .map(|grade| GradeForm {
                    name: grade.name.clone(),
                    share_percent: percent(grade.share),
                    price_per_kg: decimal(grade.price_per_kg),
                    counts_as_quality_grade: grade.counts_as_quality_grade,
                })
                .collect(),
            variable_costs: plan
                .variable_costs
                .iter()
                .map(|line| VariableCostForm {
                    name: line.name.clone(),
                    kind: line.kind,
                    quantity: decimal(line.quantity),
                    unit: line.unit.clone(),
                    unit_price: decimal(line.unit_price),
                })
                .collect(),
            fixed_costs: plan
                .fixed_costs
                .iter()
                .map(|line| FixedCostForm {
                    name: line.name.clone(),
                    cash_kind: line.cash_kind,
                    amount_per_year: decimal(line.amount_per_year),
                    investment_base: decimal(line.investment_base),
                })
                .collect(),
            targets: TargetsForm {
                yield_per_rai: decimal(plan.targets.yield_per_rai),
                yield_per_tree: decimal(plan.targets.yield_per_tree),
                yield_per_labor_day: decimal(plan.targets.yield_per_labor_day),
                yield_per_fertilizer_kg: decimal(plan.targets.yield_per_fertilizer_kg),
                yield_per_water_cubic_meter: decimal(plan.targets.yield_per_water_cubic_meter),
                yield_per_kwh: decimal(plan.targets.yield_per_kwh),
                quality_grade_percent: percent(plan.targets.quality_grade_share),
                loss_percent: percent(plan.targets.loss_share),
                cost_per_kg: decimal(plan.targets.cost_per_kg),
            },
            health_scores: HealthQuestion::ALL
                .iter()
                .map(|question| {
                    plan.health_answers
                        .iter()
                        .find(|answer| answer.question == *question)
                        .and_then(|answer| answer.score)
                        .map(|score| score.to_string())
                        .unwrap_or_default()
                })
                .collect(),
        }
    }

    pub fn to_plan(&self) -> Result<Plan, Vec<FormError>> {
        let mut errors = Vec::new();
        let market = MarketPlan {
            target_customer: optional_text(&self.market.target_customer),
            demand_kg: parse_decimal(&self.market.demand_kg, "market.demand_kg", &mut errors),
            minimum_price_per_kg: parse_decimal(
                &self.market.minimum_price_per_kg,
                "market.minimum_price_per_kg",
                &mut errors,
            ),
            sales_period: optional_text(&self.market.sales_period),
            sales_channels: parse_u32(
                &self.market.sales_channels,
                "market.sales_channels",
                &mut errors,
            ),
            largest_buyer_share: parse_percent(
                &self.market.largest_buyer_percent,
                "market.largest_buyer_share",
                &mut errors,
            ),
            quality_requirements: optional_text(&self.market.quality_requirements),
        };
        let production = ProductionPlan {
            area_rai: parse_decimal(
                &self.production.area_rai,
                "production.area_rai",
                &mut errors,
            ),
            producing_trees: parse_decimal(
                &self.production.producing_trees,
                "production.producing_trees",
                &mut errors,
            ),
            fruits_per_tree: parse_decimal(
                &self.production.fruits_per_tree,
                "production.fruits_per_tree",
                &mut errors,
            ),
            average_fruit_weight_kg: parse_decimal(
                &self.production.average_fruit_weight_kg,
                "production.average_fruit_weight_kg",
                &mut errors,
            ),
            loss_share: parse_percent(
                &self.production.loss_percent,
                "production.loss_share",
                &mut errors,
            ),
            grades: self
                .grades
                .iter()
                .enumerate()
                .map(|(index, grade)| Grade {
                    name: grade.name.trim().to_owned(),
                    share: parse_percent(
                        &grade.share_percent,
                        &format!("production.grades[{index}].share"),
                        &mut errors,
                    ),
                    price_per_kg: parse_decimal(
                        &grade.price_per_kg,
                        &format!("production.grades[{index}].price_per_kg"),
                        &mut errors,
                    ),
                    counts_as_quality_grade: grade.counts_as_quality_grade,
                })
                .collect(),
        };
        let variable_costs = self
            .variable_costs
            .iter()
            .enumerate()
            .map(|(index, line)| VariableCostLine {
                name: line.name.trim().to_owned(),
                kind: line.kind,
                quantity: parse_decimal(
                    &line.quantity,
                    &format!("variable_costs[{index}].quantity"),
                    &mut errors,
                ),
                unit: line.unit.trim().to_owned(),
                unit_price: parse_decimal(
                    &line.unit_price,
                    &format!("variable_costs[{index}].unit_price"),
                    &mut errors,
                ),
            })
            .collect();
        let fixed_costs = self
            .fixed_costs
            .iter()
            .enumerate()
            .map(|(index, line)| FixedCostLine {
                name: line.name.trim().to_owned(),
                cash_kind: line.cash_kind,
                amount_per_year: parse_decimal(
                    &line.amount_per_year,
                    &format!("fixed_costs[{index}].amount_per_year"),
                    &mut errors,
                ),
                investment_base: parse_decimal(
                    &line.investment_base,
                    &format!("fixed_costs[{index}].investment_base"),
                    &mut errors,
                ),
            })
            .collect();
        let health_answers = HealthQuestion::ALL
            .iter()
            .enumerate()
            .filter_map(|(index, question)| {
                let score = parse_u8(
                    self.health_scores.get(index).map_or("", String::as_str),
                    &format!("health_answers[{index}]"),
                    &mut errors,
                );
                score.map(|score| HealthAnswer {
                    question: *question,
                    score: Some(score),
                })
            })
            .collect();
        let targets = KpiTargets {
            yield_per_rai: parse_decimal(
                &self.targets.yield_per_rai,
                "targets.yield_per_rai",
                &mut errors,
            ),
            yield_per_tree: parse_decimal(
                &self.targets.yield_per_tree,
                "targets.yield_per_tree",
                &mut errors,
            ),
            yield_per_labor_day: parse_decimal(
                &self.targets.yield_per_labor_day,
                "targets.yield_per_labor_day",
                &mut errors,
            ),
            yield_per_fertilizer_kg: parse_decimal(
                &self.targets.yield_per_fertilizer_kg,
                "targets.yield_per_fertilizer_kg",
                &mut errors,
            ),
            yield_per_water_cubic_meter: parse_decimal(
                &self.targets.yield_per_water_cubic_meter,
                "targets.yield_per_water_cubic_meter",
                &mut errors,
            ),
            yield_per_kwh: parse_decimal(
                &self.targets.yield_per_kwh,
                "targets.yield_per_kwh",
                &mut errors,
            ),
            quality_grade_share: parse_percent(
                &self.targets.quality_grade_percent,
                "targets.quality_grade_share",
                &mut errors,
            ),
            loss_share: parse_percent(
                &self.targets.loss_percent,
                "targets.loss_share",
                &mut errors,
            ),
            cost_per_kg: parse_decimal(
                &self.targets.cost_per_kg,
                "targets.cost_per_kg",
                &mut errors,
            ),
        };

        let plan = Plan {
            name: self.name.trim().to_owned(),
            market,
            production,
            variable_costs,
            fixed_costs,
            health_answers,
            targets,
        };
        errors.extend(plan.input_issues().into_iter().map(|issue| FormError {
            field: issue.field,
            message: "ค่าที่กรอกไม่ผ่านกติกาของแผน".into(),
        }));
        if errors.is_empty() {
            Ok(plan)
        } else {
            Err(errors)
        }
    }

    pub fn section_complete(&self, section: &str) -> bool {
        match section {
            "market" => !self.market.target_customer.trim().is_empty(),
            "production" => !self.production.area_rai.trim().is_empty() && !self.grades.is_empty(),
            "variable-costs" => !self.variable_costs.is_empty(),
            "fixed-costs" => !self.fixed_costs.is_empty(),
            "targets" => !self.targets.yield_per_rai.trim().is_empty(),
            "health" => {
                self.health_scores.len() == 12
                    && self.health_scores.iter().all(|score| !score.is_empty())
            }
            _ => false,
        }
    }

    pub fn grade_total_percent(&self) -> Option<Decimal> {
        self.grades.iter().try_fold(Decimal::ZERO, |total, grade| {
            Decimal::from_str(grade.share_percent.trim())
                .ok()
                .map(|share| total + share)
        })
    }
}

fn optional_text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn text(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn decimal(value: Option<Decimal>) -> String {
    value
        .map(|value| value.normalize().to_string())
        .unwrap_or_default()
}

fn percent(value: Option<Decimal>) -> String {
    decimal(value.map(|value| value * Decimal::ONE_HUNDRED))
}

fn parse_decimal(value: &str, field: &str, errors: &mut Vec<FormError>) -> Option<Decimal> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    match Decimal::from_str(&value.replace(',', "")) {
        Ok(value) => Some(value),
        Err(_) => {
            errors.push(FormError {
                field: field.into(),
                message: "กรุณากรอกเป็นตัวเลข".into(),
            });
            None
        }
    }
}

fn parse_percent(value: &str, field: &str, errors: &mut Vec<FormError>) -> Option<Decimal> {
    parse_decimal(value, field, errors).map(|value| value / Decimal::ONE_HUNDRED)
}

fn parse_u32(value: &str, field: &str, errors: &mut Vec<FormError>) -> Option<u32> {
    parse_integer(value, field, errors)
}

fn parse_u8(value: &str, field: &str, errors: &mut Vec<FormError>) -> Option<u8> {
    parse_integer(value, field, errors)
}

fn parse_integer<T: FromStr>(value: &str, field: &str, errors: &mut Vec<FormError>) -> Option<T> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    match value.parse() {
        Ok(value) => Some(value),
        Err(_) => {
            errors.push(FormError {
                field: field.into(),
                message: "กรุณากรอกเป็นจำนวนเต็ม".into(),
            });
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workbook_sample_round_trips_through_the_form_without_loss() {
        let plan = calc::workbook_sample();
        assert_eq!(PlanForm::from_plan(&plan).to_plan(), Ok(plan));
    }

    #[test]
    fn ten_grades_map_and_must_total_one_hundred_percent() {
        let mut form = PlanForm::from_plan(&calc::workbook_sample());
        form.grades = (1..=10)
            .map(|index| GradeForm {
                name: format!("เกรด {index}"),
                share_percent: "10".into(),
                price_per_kg: index.to_string(),
                counts_as_quality_grade: index <= 2,
            })
            .collect();
        assert_eq!(form.grade_total_percent(), Some(Decimal::ONE_HUNDRED));
        assert_eq!(
            form.to_plan().expect("valid mix").production.grades.len(),
            10
        );

        form.grades[9].share_percent = "9".into();
        let errors = form.to_plan().expect_err("99 percent must be rejected");
        assert!(
            errors
                .iter()
                .any(|error| error.field == "production.grades")
        );
    }

    #[test]
    fn invalid_typed_value_is_retained_for_correction() {
        let mut form = PlanForm::from_plan(&calc::workbook_sample());
        form.production.area_rai = "สิบไร่".into();
        assert!(form.to_plan().is_err());
        assert_eq!(form.production.area_rai, "สิบไร่");
    }

    #[test]
    fn empty_plan_remains_empty_and_usable() {
        let form = PlanForm::from_plan(&Plan::default());
        let plan = form
            .to_plan()
            .expect("missing values are distinct from invalid values");
        assert_eq!(plan, Plan::default());
        assert!(form.health_scores.iter().all(String::is_empty));
    }

    #[test]
    fn section_state_is_derived_from_owner_input() {
        let sample = PlanForm::from_plan(&calc::workbook_sample());
        for section in [
            "market",
            "production",
            "variable-costs",
            "fixed-costs",
            "targets",
            "health",
        ] {
            assert!(sample.section_complete(section), "{section}");
            assert!(!PlanForm::default().section_complete(section), "{section}");
        }
    }
}
