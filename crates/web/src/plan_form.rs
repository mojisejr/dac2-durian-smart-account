use std::str::FromStr;

use calc::{
    CashKind, FixedCostLine, Grade, HealthAnswer, HealthQuestion, InputIssueKind, KpiTargets,
    MarketPlan, Plan, PriceSource, ProductionPlan, VariableCostKind, VariableCostLine, YieldSource,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanForm {
    pub name: String,
    pub market: MarketForm,
    pub production: ProductionForm,
    /// Which unit the owner is typing grade quantities in. The stored share
    /// is always a fraction; this only decides which text field is read.
    #[serde(default)]
    pub grade_entry: GradeEntry,
    pub grades: Vec<GradeForm>,
    pub variable_costs: Vec<VariableCostForm>,
    pub fixed_costs: Vec<FixedCostForm>,
    pub targets: TargetsForm,
    pub health_scores: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MarketForm {
    pub target_customer: String,
    pub buyer_committed_kg: String,
    pub minimum_price_per_kg: String,
    pub sales_period: String,
    pub sales_channels: String,
    pub largest_buyer_percent: String,
    pub quality_requirements: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProductionForm {
    #[serde(default)]
    pub yield_source: YieldSource,
    #[serde(default)]
    pub sellable_yield_kg: String,
    pub area_rai: String,
    pub producing_trees: String,
    pub fruits_per_tree: String,
    pub average_fruit_weight_kg: String,
    pub loss_percent: String,
    #[serde(default)]
    pub price_source: PriceSource,
    #[serde(default)]
    pub average_price_per_kg: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum GradeEntry {
    #[default]
    Percent,
    Kilograms,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GradeForm {
    pub name: String,
    pub share_percent: String,
    /// Kilograms for this grade, read only while `grade_entry` is
    /// `Kilograms`; converted to a share against sellable kilograms.
    #[serde(default)]
    pub share_kg: String,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadinessTone {
    Ready,
    Missing,
    Optional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectionReadiness {
    pub label: &'static str,
    pub tone: ReadinessTone,
}

impl PlanForm {
    pub fn from_plan(plan: &Plan) -> Self {
        let sellable_yield_kg = plan.production.selected_sellable_yield_kg();
        Self {
            name: plan.name.clone(),
            market: MarketForm {
                target_customer: text(&plan.market.target_customer),
                buyer_committed_kg: decimal(plan.market.buyer_committed_kg),
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
                yield_source: plan.production.yield_source,
                sellable_yield_kg: decimal(plan.production.sellable_yield_kg),
                area_rai: decimal(plan.production.area_rai),
                producing_trees: decimal(plan.production.producing_trees),
                fruits_per_tree: decimal(plan.production.fruits_per_tree),
                average_fruit_weight_kg: decimal(plan.production.average_fruit_weight_kg),
                loss_percent: percent(plan.production.loss_share),
                price_source: plan.production.price_source,
                average_price_per_kg: decimal(plan.production.average_price_per_kg),
            },
            grade_entry: GradeEntry::Percent,
            grades: plan
                .production
                .grades
                .iter()
                .map(|grade| GradeForm {
                    name: grade.name.clone(),
                    share_percent: percent(grade.share),
                    share_kg: decimal(grade.share.zip(sellable_yield_kg).map(|(s, kg)| s * kg)),
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
            buyer_committed_kg: parse_decimal(
                &self.market.buyer_committed_kg,
                "market.buyer_committed_kg",
                &mut errors,
            ),
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
        let mut production = ProductionPlan {
            yield_source: self.production.yield_source,
            sellable_yield_kg: parse_decimal(
                &self.production.sellable_yield_kg,
                "production.sellable_yield_kg",
                &mut errors,
            ),
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
            price_source: self.production.price_source,
            average_price_per_kg: parse_decimal(
                &self.production.average_price_per_kg,
                "production.average_price_per_kg",
                &mut errors,
            ),
            grades: Vec::new(),
        };
        // Kilogram entry converts against the selected sellable figure; the
        // conversion is visible beside the field, never silent, and it stays
        // unavailable while that figure is unknown or zero.
        let sellable_yield_kg = production
            .selected_sellable_yield_kg()
            .filter(|kg| !kg.is_zero());
        production.grades = self
            .grades
            .iter()
            .enumerate()
            .map(|(index, grade)| Grade {
                name: grade.name.trim().to_owned(),
                share: match self.grade_entry {
                    GradeEntry::Percent => parse_percent(
                        &grade.share_percent,
                        &format!("production.grades[{index}].share"),
                        &mut errors,
                    ),
                    GradeEntry::Kilograms => {
                        let field = format!("production.grades[{index}].share_kg");
                        let kg = parse_decimal(&grade.share_kg, &field, &mut errors);
                        match (kg, sellable_yield_kg) {
                            (Some(kg), Some(total)) => Some(kg / total),
                            (Some(_), None) => {
                                errors.push(FormError {
                                    field,
                                    message: GRADE_KG_NEEDS_SELLABLE.into(),
                                });
                                None
                            }
                            (None, _) => None,
                        }
                    }
                },
                price_per_kg: parse_decimal(
                    &grade.price_per_kg,
                    &format!("production.grades[{index}].price_per_kg"),
                    &mut errors,
                ),
                counts_as_quality_grade: grade.counts_as_quality_grade,
            })
            .collect();
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
        errors.extend(plan.input_issues().into_iter().map(|issue| {
            FormError {
                field: issue.field,
                message: match issue.kind {
                    InputIssueKind::Negative => "กรุณากรอกตัวเลขตั้งแต่ 0 ขึ้นไป",
                    InputIssueKind::OutsideShareRange => "กรุณากรอกสัดส่วนระหว่าง 0 ถึง 100%",
                    InputIssueKind::GradeSharesDoNotTotalOne => match self.grade_entry {
                        GradeEntry::Percent => "สัดส่วนทุกเกรดรวมกันต้องเท่ากับ 100%",
                        GradeEntry::Kilograms => "กิโลกรัมทุกเกรดรวมกันต้องเท่ากับกิโลที่คาดว่าจะขายได้",
                    },
                    InputIssueKind::HealthScoreOutsideRange => "กรุณาเลือกคะแนนระหว่าง 1 ถึง 5",
                    InputIssueKind::DuplicateHealthAnswer => {
                        "คำถามข้อนี้มีคำตอบซ้ำ กรุณาเลือกเพียงคำตอบเดียว"
                    }
                }
                .into(),
            }
        }));
        if errors.is_empty() {
            Ok(plan)
        } else {
            Err(errors)
        }
    }

    pub fn section_complete(&self, section: &str) -> bool {
        self.section_readiness(section).tone == ReadinessTone::Ready
    }

    pub fn section_readiness(&self, section: &str) -> SectionReadiness {
        let Ok(plan) = self.to_plan() else {
            return SectionReadiness {
                label: "มีข้อมูลที่ต้องตรวจในส่วนนี้",
                tone: ReadinessTone::Missing,
            };
        };
        let analysis = calc::analyze(&plan);
        match section {
            "market" if plan.market.buyer_committed_kg.is_none() => SectionReadiness {
                label: "เพิ่มได้ ถ้ามียอดที่ผู้ซื้ออยากได้",
                tone: ReadinessTone::Optional,
            },
            "market" if analysis.revenue.sellable_yield_kg.is_none() => SectionReadiness {
                label: "ยังขาดผลผลิตขายได้เพื่อเทียบยอดผู้ซื้อ",
                tone: ReadinessTone::Missing,
            },
            "market" if analysis.revenue.market_fulfillment.is_some() => SectionReadiness {
                label: "พร้อมเทียบยอดผู้ซื้อกับผลผลิต",
                tone: ReadinessTone::Ready,
            },
            "market" => SectionReadiness {
                label: "ตรวจยอดที่ผู้ซื้ออยากได้",
                tone: ReadinessTone::Missing,
            },
            "production" if analysis.revenue.sellable_yield_kg.is_none() => SectionReadiness {
                label: "ยังขาดผลผลิตที่ขายได้",
                tone: ReadinessTone::Missing,
            },
            "production" if analysis.revenue.weighted_price_per_kg.is_none() => SectionReadiness {
                label: "ยังขาดราคาขายเฉลี่ย",
                tone: ReadinessTone::Missing,
            },
            "production" => SectionReadiness {
                label: "พอคำนวณรายได้แล้ว",
                tone: ReadinessTone::Ready,
            },
            "variable-costs" if analysis.cost.variable_cost.is_some() => SectionReadiness {
                label: "พอคำนวณค่าใช้จ่ายตามการผลิตแล้ว",
                tone: ReadinessTone::Ready,
            },
            "variable-costs" => SectionReadiness {
                label: "ยังขาดค่าใช้จ่ายตามการผลิต",
                tone: ReadinessTone::Missing,
            },
            "fixed-costs" if analysis.cost.fixed_cost.is_some() => SectionReadiness {
                label: "พอคำนวณค่าใช้จ่ายประจำแล้ว",
                tone: ReadinessTone::Ready,
            },
            "fixed-costs" => SectionReadiness {
                label: "ยังขาดค่าใช้จ่ายประจำ",
                tone: ReadinessTone::Missing,
            },
            "targets" if self.targets.yield_per_rai.trim().is_empty() => SectionReadiness {
                label: "เพิ่มได้เมื่ออยากตั้งเป้าหมายเอง",
                tone: ReadinessTone::Optional,
            },
            "targets" => SectionReadiness {
                label: "มีเป้าหมายสำหรับเปรียบเทียบแล้ว",
                tone: ReadinessTone::Ready,
            },
            "health"
                if self.health_scores.len() == 12
                    && self.health_scores.iter().all(|score| !score.is_empty()) =>
            {
                SectionReadiness {
                    label: "พร้อมดูแบบประเมินสวน",
                    tone: ReadinessTone::Ready,
                }
            }
            "health" => SectionReadiness {
                label: "เพิ่มได้เพื่อทบทวนความพร้อมของสวน",
                tone: ReadinessTone::Optional,
            },
            _ => SectionReadiness {
                label: "ยังไม่มีสถานะ",
                tone: ReadinessTone::Missing,
            },
        }
    }

    pub fn section_errors(&self, section: &str) -> Vec<FormError> {
        self.to_plan()
            .err()
            .unwrap_or_default()
            .into_iter()
            .filter(|error| field_belongs_to_section(&error.field, section))
            .collect()
    }

    pub fn replace_section_from(&mut self, section: &str, submitted: &Self) -> bool {
        match section {
            "market" => self.market = submitted.market.clone(),
            "production" => {
                self.production = submitted.production.clone();
                self.grades = submitted.grades.clone();
            }
            "variable-costs" => self.variable_costs = submitted.variable_costs.clone(),
            "fixed-costs" => self.fixed_costs = submitted.fixed_costs.clone(),
            "targets" => self.targets = submitted.targets.clone(),
            "health" => self.health_scores = submitted.health_scores.clone(),
            _ => return false,
        }
        true
    }

    pub fn grade_total_percent(&self) -> Option<Decimal> {
        self.grades.iter().try_fold(Decimal::ZERO, |total, grade| {
            Decimal::from_str(grade.share_percent.trim())
                .ok()
                .map(|share| total + share)
        })
    }

    pub fn grade_total_kg(&self) -> Option<Decimal> {
        self.grades.iter().try_fold(Decimal::ZERO, |total, grade| {
            Decimal::from_str(&grade.share_kg.trim().replace(',', ""))
                .ok()
                .map(|kg| total + kg)
        })
    }

    /// Sellable kilograms from the selected production branch as typed so
    /// far, ignoring grades; `None` while unknown or zero.
    pub fn sellable_yield_kg(&self) -> Option<Decimal> {
        let mut form = self.clone();
        form.grades.clear();
        form.to_plan()
            .ok()?
            .production
            .selected_sellable_yield_kg()
            .filter(|kg| !kg.is_zero())
    }

    /// Sellable kilograms derived from the orchard facts as typed, whichever
    /// branch is selected, so it can be shown beside a direct entry.
    pub fn derived_sellable_yield_kg(&self) -> Option<Decimal> {
        let mut form = self.clone();
        form.grades.clear();
        form.to_plan().ok()?.production.derived_sellable_yield_kg()
    }

    /// The by-grade weighted price as typed, whichever branch is selected.
    pub fn weighted_grade_price_per_kg(&self) -> Option<Decimal> {
        self.to_plan()
            .ok()?
            .production
            .weighted_grade_price_per_kg()
    }

    /// The kilogram equivalent of one grade's typed percentage.
    pub fn grade_kg_from_percent(&self, index: usize) -> Option<Decimal> {
        let share = Decimal::from_str(self.grades.get(index)?.share_percent.trim()).ok()?;
        Some(share / Decimal::ONE_HUNDRED * self.sellable_yield_kg()?)
    }

    /// The percentage equivalent of one grade's typed kilograms.
    pub fn grade_percent_from_kg(&self, index: usize) -> Option<Decimal> {
        let kg =
            Decimal::from_str(&self.grades.get(index)?.share_kg.trim().replace(',', "")).ok()?;
        Some(kg / self.sellable_yield_kg()? * Decimal::ONE_HUNDRED)
    }

    /// Switch the grade entry unit, filling the other unit's field from the
    /// typed one so the conversion is visible. Kilogram entry needs a known,
    /// non-zero sellable figure; without it the switch is refused.
    pub fn set_grade_entry(&mut self, entry: GradeEntry) -> bool {
        if entry == self.grade_entry {
            return true;
        }
        let Some(total) = self.sellable_yield_kg() else {
            return entry == GradeEntry::Percent && {
                self.grade_entry = entry;
                true
            };
        };
        for grade in &mut self.grades {
            match entry {
                GradeEntry::Kilograms => {
                    if let Ok(share) = Decimal::from_str(grade.share_percent.trim()) {
                        grade.share_kg =
                            decimal(Some((share / Decimal::ONE_HUNDRED * total).round_dp(2)));
                    }
                }
                GradeEntry::Percent => {
                    if let Ok(kg) = Decimal::from_str(&grade.share_kg.trim().replace(',', "")) {
                        grade.share_percent =
                            decimal(Some((kg / total * Decimal::ONE_HUNDRED).round_dp(2)));
                    }
                }
            }
        }
        self.grade_entry = entry;
        true
    }
}

pub const GRADE_KG_NEEDS_SELLABLE: &str = "ยังขาดกิโลที่คาดว่าจะขายได้ จึงแปลงกิโลกรัมของเกรดเป็นสัดส่วนไม่ได้";

fn field_belongs_to_section(field: &str, section: &str) -> bool {
    match section {
        "market" => field.starts_with("market."),
        "production" => field.starts_with("production."),
        "variable-costs" => field.starts_with("variable_costs["),
        "fixed-costs" => field.starts_with("fixed_costs["),
        "targets" => field.starts_with("targets."),
        "health" => field.starts_with("health_answers"),
        _ => false,
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
                share_kg: String::new(),
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

    #[test]
    fn readiness_names_the_result_instead_of_claiming_generic_completeness() {
        let sample = PlanForm::from_plan(&calc::workbook_sample());
        assert_eq!(
            sample.section_readiness("production"),
            SectionReadiness {
                label: "พอคำนวณรายได้แล้ว",
                tone: ReadinessTone::Ready,
            }
        );
        assert_eq!(
            PlanForm::default().section_readiness("market").tone,
            ReadinessTone::Optional
        );
        assert_eq!(
            PlanForm::default().section_readiness("production").label,
            "ยังขาดผลผลิตที่ขายได้"
        );

        let mut partial = sample.clone();
        for grade in &mut partial.grades {
            grade.price_per_kg.clear();
        }
        assert_eq!(
            partial.section_readiness("production"),
            SectionReadiness {
                label: "ยังขาดราคาขายเฉลี่ย",
                tone: ReadinessTone::Missing,
            }
        );

        let mut without_buyer_commitment = sample;
        without_buyer_commitment.market.buyer_committed_kg.clear();
        assert_eq!(
            without_buyer_commitment.section_readiness("market").tone,
            ReadinessTone::Optional
        );
    }

    #[test]
    fn section_validation_ignores_invalid_values_outside_the_visible_section() {
        let mut form = PlanForm::from_plan(&calc::workbook_sample());
        form.production.area_rai = "ไม่ใช่ตัวเลข".into();

        assert!(form.to_plan().is_err());
        assert!(form.section_errors("market").is_empty());
        assert_eq!(
            form.section_errors("production")[0].message,
            "กรุณากรอกเป็นตัวเลข"
        );
    }

    #[test]
    fn replacing_one_section_preserves_every_other_section() {
        let mut stored = PlanForm::from_plan(&calc::workbook_sample());
        let original_production = stored.production.clone();
        let original_grades = stored.grades.clone();
        let mut submitted = stored.clone();
        submitted.market.target_customer = "ตลาดหน้าสวน".into();
        submitted.production.area_rai = "ไม่ใช่ตัวเลข".into();

        assert!(stored.replace_section_from("market", &submitted));
        assert_eq!(stored.market.target_customer, "ตลาดหน้าสวน");
        assert_eq!(stored.production, original_production);
        assert_eq!(stored.grades, original_grades);
        assert!(!stored.replace_section_from("unknown", &submitted));
    }

    #[test]
    fn semantic_errors_explain_how_to_correct_the_value() {
        let mut form = PlanForm::from_plan(&calc::workbook_sample());
        form.production.area_rai = "-1".into();
        let errors = form.section_errors("production");
        assert!(
            errors
                .iter()
                .any(|error| error.message == "กรุณากรอกตัวเลขตั้งแต่ 0 ขึ้นไป")
        );
    }

    fn plan_with_costs_only() -> calc::Plan {
        let mut plan = calc::workbook_sample();
        plan.market = calc::MarketPlan::default();
        plan.production = calc::ProductionPlan::default();
        plan
    }

    fn net_profit(form: &PlanForm) -> Option<Decimal> {
        calc::analyze(&form.to_plan().expect("fixture is valid"))
            .business
            .net_profit
    }

    #[test]
    fn total_kg_and_one_price_is_enough_for_the_main_result() {
        let mut form = PlanForm::from_plan(&plan_with_costs_only());
        form.production.yield_source = YieldSource::Direct;
        form.production.sellable_yield_kg = "19950".into();
        form.production.price_source = PriceSource::Average;
        form.production.average_price_per_kg = "82.5".into();

        assert!(net_profit(&form).is_some());
        assert_eq!(
            form.section_readiness("production").tone,
            ReadinessTone::Ready
        );
        assert_eq!(
            form.section_readiness("market").label,
            "เพิ่มได้ ถ้ามียอดที่ผู้ซื้ออยากได้",
            "market comparison is optional and names what would unlock it"
        );
    }

    #[test]
    fn tree_facts_without_grades_name_the_missing_price_not_the_yield() {
        let mut form = PlanForm::from_plan(&plan_with_costs_only());
        form.production.producing_trees = "200".into();
        form.production.fruits_per_tree = "35".into();
        form.production.average_fruit_weight_kg = "3".into();
        form.production.loss_percent = "5".into();
        form.production.price_source = PriceSource::ByGrade;

        assert_eq!(form.sellable_yield_kg(), Some(Decimal::from(19_950)));
        assert!(net_profit(&form).is_none());
        assert_eq!(
            form.section_readiness("production").label,
            "ยังขาดราคาขายเฉลี่ย"
        );

        form.production.price_source = PriceSource::Average;
        form.production.average_price_per_kg = "80".into();
        assert!(
            net_profit(&form).is_some(),
            "an unknown grade mix never blocks the result once one average price exists"
        );
    }

    #[test]
    fn grade_sales_known_in_kilograms_reach_the_result_and_the_stored_share() {
        let mut form = PlanForm::from_plan(&plan_with_costs_only());
        form.production.yield_source = YieldSource::Direct;
        form.production.sellable_yield_kg = "20000".into();
        form.production.price_source = PriceSource::ByGrade;
        form.grade_entry = GradeEntry::Kilograms;
        form.grades = vec![
            GradeForm {
                name: "A".into(),
                share_kg: "15000".into(),
                price_per_kg: "100".into(),
                counts_as_quality_grade: true,
                ..GradeForm::default()
            },
            GradeForm {
                name: "B".into(),
                share_kg: "5000".into(),
                price_per_kg: "60".into(),
                counts_as_quality_grade: false,
                ..GradeForm::default()
            },
        ];

        let plan = form.to_plan().expect("kilogram entry converts to shares");
        assert_eq!(plan.production.grades[0].share, Some(Decimal::new(75, 2)));
        assert_eq!(plan.production.grades[1].share, Some(Decimal::new(25, 2)));
        assert_eq!(
            form.grade_percent_from_kg(0),
            Some(Decimal::from(75)),
            "the converted percentage is available to show beside the entry"
        );
        assert!(net_profit(&form).is_some());

        form.grades[1].share_kg = "4000".into();
        let errors = form
            .to_plan()
            .expect_err("kilograms must add up to the total");
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("กิโลกรัมทุกเกรดรวมกัน"))
        );
    }

    #[test]
    fn buyer_quantity_unknown_leaves_the_result_available_and_market_optional() {
        let mut form = PlanForm::from_plan(&calc::workbook_sample());
        form.market.buyer_committed_kg.clear();

        assert!(net_profit(&form).is_some());
        assert_eq!(
            form.section_readiness("market").tone,
            ReadinessTone::Optional
        );
        let analysis = calc::analyze(&form.to_plan().expect("valid"));
        assert_eq!(analysis.revenue.market_fulfillment, None);
        assert_eq!(analysis.revenue.market_gap_kg, None);
    }

    #[test]
    fn kilogram_entry_is_unavailable_with_a_named_reason_until_sellable_is_known() {
        let mut form = PlanForm::from_plan(&calc::Plan::default());
        form.grades.push(GradeForm {
            name: "A".into(),
            share_percent: "50".into(),
            ..GradeForm::default()
        });

        assert!(!form.set_grade_entry(GradeEntry::Kilograms));
        assert_eq!(form.grade_entry, GradeEntry::Percent);

        form.grade_entry = GradeEntry::Kilograms;
        form.grades[0].share_kg = "100".into();
        let errors = form.to_plan().expect_err("kilograms cannot convert yet");
        assert!(
            errors
                .iter()
                .any(|error| error.message == GRADE_KG_NEEDS_SELLABLE)
        );
        assert!(
            !form.section_errors("production").is_empty(),
            "the reason belongs to the visible production section"
        );

        form.grade_entry = GradeEntry::Percent;
        form.production.yield_source = YieldSource::Direct;
        form.production.sellable_yield_kg = "0".into();
        assert!(
            !form.set_grade_entry(GradeEntry::Kilograms),
            "a zero total cannot convert either"
        );
    }

    #[test]
    fn switching_the_grade_unit_converts_visibly_in_both_directions() {
        let mut form = PlanForm::from_plan(&calc::workbook_sample());
        assert_eq!(form.grade_entry, GradeEntry::Percent);
        assert_eq!(
            form.grade_kg_from_percent(0),
            Some(Decimal::from(9_975)),
            "50% of 19,950 kg is shown beside the percentage"
        );

        assert!(form.set_grade_entry(GradeEntry::Kilograms));
        assert_eq!(form.grades[0].share_kg, "9975");
        assert_eq!(form.grades[3].share_kg, "997.5");
        assert_eq!(
            form.to_plan()
                .expect("kilograms convert back to the same shares"),
            calc::workbook_sample()
        );

        form.grades[0].share_kg = "11970".into();
        form.grades[1].share_kg = "3990".into();
        assert!(form.set_grade_entry(GradeEntry::Percent));
        assert_eq!(form.grades[0].share_percent, "60");
        assert_eq!(form.grades[1].share_percent, "20");
    }

    #[test]
    fn the_unselected_branch_survives_the_form_round_trip() {
        let mut plan = calc::workbook_sample();
        plan.production.yield_source = YieldSource::Direct;
        plan.production.sellable_yield_kg = Some(Decimal::from(18_000));
        plan.production.price_source = PriceSource::Average;
        plan.production.average_price_per_kg = Some(Decimal::from(79));

        let form = PlanForm::from_plan(&plan);
        assert_eq!(form.production.producing_trees, "200");
        assert_eq!(form.grades.len(), 4);
        assert_eq!(form.to_plan(), Ok(plan));
        assert_eq!(
            form.derived_sellable_yield_kg(),
            Some(Decimal::from(19_950)),
            "the derived figure can be shown beside the direct entry"
        );
        assert_eq!(
            form.weighted_grade_price_per_kg(),
            Some(Decimal::new(825, 1)),
            "the grade price can be shown beside the average entry"
        );
    }
}
