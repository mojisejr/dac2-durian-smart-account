use std::str::FromStr;

use calc::{
    CashKind, CostSectionState, FixedCostLine, Grade, HealthAnswer, HealthQuestion, InputIssueKind,
    KpiTargets, MarketPlan, Plan, PriceSource, ProductionPlan, TaxDeductionLine,
    UnclassifiedExpense, VariableCostKind, VariableCostLine, YieldSource,
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
    #[serde(default)]
    pub variable_cost_state: CostSectionState,
    pub variable_costs: Vec<VariableCostForm>,
    #[serde(default)]
    pub fixed_cost_state: CostSectionState,
    pub fixed_costs: Vec<FixedCostForm>,
    #[serde(default)]
    pub unclassified: Vec<UnclassifiedExpenseForm>,
    /// Deductions the owner claims for the season, one row each. Never
    /// pre-filled; an empty list is "not yet entered".
    #[serde(default)]
    pub tax_deductions: Vec<TaxDeductionForm>,
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
    /// The owner knows only the total for this line. Only the selected way
    /// of answering is read; the other fields are kept but not used.
    #[serde(default)]
    pub total_only: bool,
    #[serde(default)]
    pub total_amount: String,
}

impl Default for VariableCostForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: VariableCostKind::Other,
            quantity: String::new(),
            unit: String::new(),
            unit_price: String::new(),
            total_only: false,
            total_amount: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UnclassifiedExpenseForm {
    pub name: String,
    pub amount: String,
    pub note: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TaxDeductionForm {
    pub name: String,
    pub amount: String,
}

/// How a remembered expense is filed once the owner answers the familiar
/// questions. Variable lines take the amount as a total; fixed lines take it
/// as the amount per year.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpenseClassification {
    Variable(VariableCostKind),
    Fixed(CashKind),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionReadiness {
    pub label: std::borrow::Cow<'static, str>,
    pub tone: ReadinessTone,
}

/// The questions an owner opens this application to answer. Readiness is
/// named per decision, not per input section: a page says which of these
/// can be answered now and which single question is still missing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    FirstEstimate,
    MarketComparison,
    CashView,
    InvestmentView,
    HealthSelfReview,
    FinalClose,
}

impl Decision {
    pub const ALL: [Decision; 6] = [
        Decision::FirstEstimate,
        Decision::MarketComparison,
        Decision::CashView,
        Decision::InvestmentView,
        Decision::HealthSelfReview,
        Decision::FinalClose,
    ];

    /// The decision in the owner's words.
    pub fn question(self) -> &'static str {
        match self {
            Decision::FirstEstimate => "ปีนี้จะเหลือกำไรเท่าไร",
            Decision::MarketComparison => "ของที่มีพอกับที่ผู้ซื้อคุยไว้ไหม",
            Decision::CashView => "ปีนี้เงินสดจะเหลือเท่าไร",
            Decision::InvestmentView => "คุ้มกับเงินก้อนที่ลงไปไหม",
            Decision::HealthSelfReview => "สวนพร้อมแค่ไหน ตามที่ประเมินเอง",
            Decision::FinalClose => "ปิดฤดูด้วยตัวเลขจริง",
        }
    }

    /// The accounting or planning term the decision is known by.
    pub fn formal_term(self) -> &'static str {
        match self {
            Decision::FirstEstimate => "กำไรสุทธิ · จุดคุ้มทุน",
            Decision::MarketComparison => "การตอบสนองตลาด",
            Decision::CashView => "กระแสเงินสดจากการดำเนินงาน",
            Decision::InvestmentView => "ผลตอบแทนต่อเงินลงทุน (ROI) · ระยะคืนทุน",
            Decision::HealthSelfReview => "คะแนนสุขภาพธุรกิจ",
            Decision::FinalClose => "ผลจริงเทียบประมาณการ",
        }
    }
}

/// What stands between the owner and one decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionState {
    /// The result can be shown from what is entered.
    Ready,
    /// A required fact is absent; `question` names it and `section` is the
    /// page that asks it.
    Missing {
        question: &'static str,
        section: &'static str,
    },
    /// The main results do not need this; `unlock` names what would add it.
    Optional {
        unlock: &'static str,
        section: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecisionReadiness {
    pub decision: Decision,
    pub state: DecisionState,
}

impl DecisionReadiness {
    pub fn is_ready(&self) -> bool {
        self.state == DecisionState::Ready
    }
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
            variable_cost_state: plan.variable_cost_state,
            variable_costs: plan
                .variable_costs
                .iter()
                .map(|line| VariableCostForm {
                    name: line.name.clone(),
                    kind: line.kind,
                    quantity: decimal(line.quantity),
                    unit: line.unit.clone(),
                    unit_price: decimal(line.unit_price),
                    total_only: line.is_total_only(),
                    total_amount: decimal(line.total_amount),
                })
                .collect(),
            fixed_cost_state: plan.fixed_cost_state,
            unclassified: plan
                .unclassified_expenses
                .iter()
                .map(|expense| UnclassifiedExpenseForm {
                    name: expense.name.clone(),
                    amount: decimal(expense.amount),
                    note: expense.note.clone(),
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
            tax_deductions: plan
                .tax_deductions
                .iter()
                .map(|line| TaxDeductionForm {
                    name: line.name.clone(),
                    amount: decimal(Some(line.amount)),
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
            .map(|(index, line)| {
                if line.total_only {
                    VariableCostLine {
                        name: line.name.trim().to_owned(),
                        kind: line.kind,
                        quantity: None,
                        unit: String::new(),
                        unit_price: None,
                        total_amount: parse_decimal(
                            &line.total_amount,
                            &format!("variable_costs[{index}].total_amount"),
                            &mut errors,
                        ),
                    }
                } else {
                    VariableCostLine {
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
                        total_amount: None,
                    }
                }
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

        let unclassified_expenses = self
            .unclassified
            .iter()
            .enumerate()
            .map(|(index, expense)| UnclassifiedExpense {
                name: expense.name.trim().to_owned(),
                amount: parse_decimal(
                    &expense.amount,
                    &format!("unclassified_expenses[{index}].amount"),
                    &mut errors,
                ),
                note: expense.note.trim().to_owned(),
            })
            .collect();
        // A deduction row needs both a name and an amount; a row the owner
        // added and left blank is dropped rather than saved as a zero, and a
        // half-filled row is an error on the field that is missing.
        let tax_deductions = self
            .tax_deductions
            .iter()
            .enumerate()
            .filter(|(_, line)| !(line.name.trim().is_empty() && line.amount.trim().is_empty()))
            .filter_map(|(index, line)| {
                let name = line.name.trim().to_owned();
                if name.is_empty() {
                    errors.push(FormError {
                        field: format!("tax_deductions[{index}].name"),
                        message: "กรุณาใส่ชื่อรายการลดหย่อน".into(),
                    });
                }
                let amount = match parse_decimal(
                    &line.amount,
                    &format!("tax_deductions[{index}].amount"),
                    &mut errors,
                ) {
                    Some(amount) if amount < Decimal::ZERO => {
                        errors.push(FormError {
                            field: format!("tax_deductions[{index}].amount"),
                            message: "จำนวนเงินต้องไม่ติดลบ".into(),
                        });
                        None
                    }
                    Some(amount) => Some(amount),
                    None => {
                        if line.amount.trim().is_empty() {
                            errors.push(FormError {
                                field: format!("tax_deductions[{index}].amount"),
                                message: "กรุณาใส่จำนวนเงินที่ลดหย่อนได้".into(),
                            });
                        }
                        None
                    }
                };
                (!name.is_empty())
                    .then(|| amount.map(|amount| TaxDeductionLine { name, amount }))
                    .flatten()
            })
            .collect();
        let plan = Plan {
            name: self.name.trim().to_owned(),
            market,
            production,
            variable_cost_state: self.variable_cost_state,
            variable_costs,
            fixed_cost_state: self.fixed_cost_state,
            fixed_costs,
            tax_deductions,
            unclassified_expenses,
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
                    InputIssueKind::TotalAndUnitEntered => {
                        "รายการนี้กรอกทั้งยอดรวมและจำนวนกับราคาต่อหน่วย กรุณาเลือกกรอกแบบเดียว"
                    }
                    InputIssueKind::ConfirmedNoneWithRows => {
                        "ส่วนนี้มีรายการอยู่ จึงยืนยันว่าไม่มีไม่ได้ ลบรายการก่อนหรือเปลี่ยนคำตอบ"
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

    /// Readiness that knows about assets already included in the season, so
    /// an unknown fixed section says the depreciation is counted and names
    /// what is still missing instead of reading as if nothing exists.
    pub fn section_readiness_with_assets(
        &self,
        section: &str,
        asset_depreciation: Option<Decimal>,
    ) -> SectionReadiness {
        let readiness = self.section_readiness(section);
        match (section, asset_depreciation, readiness.tone) {
            ("fixed-costs", Some(_), ReadinessTone::Missing)
                if self.fixed_costs.is_empty()
                    && self.fixed_cost_state != CostSectionState::ConfirmedNone =>
            {
                SectionReadiness {
                    label: "ค่าเสื่อมของที่เลือกไว้รวมแล้ว ยังต้องกรอกหรือยืนยันว่าไม่มีค่าใช้จ่ายประจำอื่น".into(),
                    tone: ReadinessTone::Missing,
                }
            }
            ("fixed-costs", Some(_), ReadinessTone::Ready)
                if self.fixed_cost_state == CostSectionState::ConfirmedNone =>
            {
                SectionReadiness {
                    label: "ยืนยันแล้วว่ามีเฉพาะค่าเสื่อมของที่เลือกไว้".into(),
                    tone: ReadinessTone::Ready,
                }
            }
            _ => readiness,
        }
    }

    /// One readiness statement per decision, from the same analysis the
    /// dashboard shows, so the hub, the dashboard, and the analysis page
    /// cannot disagree about what can be answered.
    pub fn decision_readiness(
        &self,
        assets: &[calc::AssetAllocation],
        starting_capital: Option<Decimal>,
    ) -> Vec<DecisionReadiness> {
        let analysis = self
            .to_plan()
            .ok()
            .map(|plan| calc::analyze_with_assets(&plan, assets, starting_capital));
        Decision::ALL
            .into_iter()
            .map(|decision| DecisionReadiness {
                decision,
                state: self.decision_state(decision, analysis.as_ref()),
            })
            .collect()
    }

    fn decision_state(
        &self,
        decision: Decision,
        analysis: Option<&calc::Analysis>,
    ) -> DecisionState {
        use DecisionState::{Missing, Optional, Ready};
        let Some(analysis) = analysis else {
            return match decision {
                Decision::FinalClose => Ready,
                Decision::HealthSelfReview => self.health_state(),
                _ => Missing {
                    question: "มีข้อมูลที่ต้องตรวจก่อน",
                    section: "production",
                },
            };
        };
        // The first missing fact on the road to a profit figure, in the
        // order the guided path asks them.
        let first_estimate_gap = if analysis.revenue.sellable_yield_kg.is_none() {
            Some(("ปีนี้จะขายได้กี่กิโล", "production"))
        } else if analysis.revenue.weighted_price_per_kg.is_none() {
            Some(("ขายกิโลละเท่าไร", "production"))
        } else if !self.unclassified.is_empty() {
            Some(("ค่าใช้จ่ายที่จดไว้เป็นแบบไหน", "expenses"))
        } else if analysis.cost.variable_cost.is_none() {
            Some(("ค่าใช้จ่ายที่เพิ่มตามการผลิต หรือยืนยันว่าไม่มี", "variable-costs"))
        } else if analysis.cost.fixed_cost.is_none() {
            Some(("ค่าใช้จ่ายประจำ หรือยืนยันว่าไม่มี", "fixed-costs"))
        } else {
            None
        };
        let gap = |gap: Option<(&'static str, &'static str)>| match gap {
            Some((question, section)) => Missing { question, section },
            None => Ready,
        };
        match decision {
            Decision::FirstEstimate => gap(first_estimate_gap),
            Decision::MarketComparison => {
                if analysis.revenue.market_fulfillment.is_some() {
                    Ready
                } else if self.market.buyer_committed_kg.trim().is_empty() {
                    Optional {
                        unlock: "ยอดที่ผู้ซื้อคุยว่าจะรับ",
                        section: "market",
                    }
                } else {
                    Missing {
                        question: "ปีนี้จะขายได้กี่กิโล",
                        section: "production",
                    }
                }
            }
            Decision::CashView => {
                if analysis.business.operating_cash_flow.is_some() {
                    Ready
                } else {
                    gap(first_estimate_gap)
                }
            }
            Decision::InvestmentView => {
                if analysis.business.roi.is_some() {
                    Ready
                } else if first_estimate_gap.is_some() {
                    gap(first_estimate_gap)
                } else {
                    Optional {
                        unlock: "เงินก้อนที่ลงไป ของที่ใช้หลายปี หรือเงินก้อนตั้งต้น",
                        section: "assets",
                    }
                }
            }
            Decision::HealthSelfReview => self.health_state(),
            Decision::FinalClose => Ready,
        }
    }

    fn health_state(&self) -> DecisionState {
        if self.health_scores.len() == 12
            && self.health_scores.iter().all(|score| !score.is_empty())
        {
            DecisionState::Ready
        } else {
            DecisionState::Optional {
                unlock: "แบบประเมินสวน 12 ข้อ",
                section: "health",
            }
        }
    }

    pub fn section_readiness(&self, section: &str) -> SectionReadiness {
        let Ok(plan) = self.to_plan() else {
            return SectionReadiness {
                label: "มีข้อมูลที่ต้องตรวจในส่วนนี้".into(),
                tone: ReadinessTone::Missing,
            };
        };
        let analysis = calc::analyze(&plan);
        match section {
            "market" if plan.market.buyer_committed_kg.is_none() => SectionReadiness {
                label: "เพิ่มได้ ถ้ามียอดที่ผู้ซื้ออยากได้".into(),
                tone: ReadinessTone::Optional,
            },
            "market" if analysis.revenue.sellable_yield_kg.is_none() => SectionReadiness {
                label: "ยังขาดผลผลิตขายได้เพื่อเทียบยอดผู้ซื้อ".into(),
                tone: ReadinessTone::Missing,
            },
            "market" if analysis.revenue.market_fulfillment.is_some() => SectionReadiness {
                label: "พร้อมเทียบยอดผู้ซื้อกับผลผลิต".into(),
                tone: ReadinessTone::Ready,
            },
            "market" => SectionReadiness {
                label: "ตรวจยอดที่ผู้ซื้ออยากได้".into(),
                tone: ReadinessTone::Missing,
            },
            "production" if analysis.revenue.sellable_yield_kg.is_none() => SectionReadiness {
                label: "ยังขาดผลผลิตที่ขายได้".into(),
                tone: ReadinessTone::Missing,
            },
            "production" if analysis.revenue.weighted_price_per_kg.is_none() => SectionReadiness {
                label: "ยังขาดราคาขายเฉลี่ย".into(),
                tone: ReadinessTone::Missing,
            },
            "production" => SectionReadiness {
                label: "พอคำนวณรายได้แล้ว".into(),
                tone: ReadinessTone::Ready,
            },
            "expenses" if !plan.unclassified_expenses.is_empty() => SectionReadiness {
                label: match plan.unclassified_expenses.len() {
                    1 => "มี 1 รายการยังไม่ได้บอกว่าเป็นแบบไหน",
                    2 => "มี 2 รายการยังไม่ได้บอกว่าเป็นแบบไหน",
                    3 => "มี 3 รายการยังไม่ได้บอกว่าเป็นแบบไหน",
                    _ => "มีหลายรายการยังไม่ได้บอกว่าเป็นแบบไหน",
                }
                .into(),
                tone: ReadinessTone::Missing,
            },
            "expenses" => SectionReadiness {
                label: "เพิ่มได้เมื่อจำค่าใช้จ่ายได้แต่ยังไม่รู้ประเภท".into(),
                tone: ReadinessTone::Optional,
            },
            "variable-costs" => match plan.effective_variable_cost_state() {
                CostSectionState::ConfirmedNone => SectionReadiness {
                    label: "ยืนยันแล้วว่าไม่มีค่าใช้จ่ายส่วนนี้".into(),
                    tone: ReadinessTone::Ready,
                },
                CostSectionState::EnteredItems if analysis.cost.variable_cost.is_some() => {
                    SectionReadiness {
                        label: "พอคำนวณค่าใช้จ่ายตามการผลิตแล้ว".into(),
                        tone: ReadinessTone::Ready,
                    }
                }
                CostSectionState::EnteredItems => SectionReadiness {
                    label: "มีรายการที่ยังกรอกไม่ครบ".into(),
                    tone: ReadinessTone::Missing,
                },
                CostSectionState::Unknown => SectionReadiness {
                    label: "ยังขาดค่าใช้จ่ายตามการผลิต หรือยืนยันว่าไม่มี".into(),
                    tone: ReadinessTone::Missing,
                },
            },
            "fixed-costs" => match plan.effective_fixed_cost_state() {
                CostSectionState::ConfirmedNone => SectionReadiness {
                    label: "ยืนยันแล้วว่าไม่มีค่าใช้จ่ายส่วนนี้".into(),
                    tone: ReadinessTone::Ready,
                },
                CostSectionState::EnteredItems if analysis.cost.fixed_cost.is_some() => {
                    SectionReadiness {
                        label: "พอคำนวณค่าใช้จ่ายประจำแล้ว".into(),
                        tone: ReadinessTone::Ready,
                    }
                }
                CostSectionState::EnteredItems => SectionReadiness {
                    label: "มีรายการที่ยังกรอกไม่ครบ".into(),
                    tone: ReadinessTone::Missing,
                },
                CostSectionState::Unknown => SectionReadiness {
                    label: "ยังขาดค่าใช้จ่ายประจำ หรือยืนยันว่าไม่มี".into(),
                    tone: ReadinessTone::Missing,
                },
            },
            "tax-deductions" if plan.tax_deductions.is_empty() => SectionReadiness {
                label: "ยังไม่ได้กรอก · ภาษีคิดโดยยังไม่หักลดหย่อน".into(),
                tone: ReadinessTone::Optional,
            },
            "tax-deductions" => SectionReadiness {
                label: deductions_entered_label(&plan).into(),
                tone: ReadinessTone::Ready,
            },
            "targets" if self.targets.yield_per_rai.trim().is_empty() => SectionReadiness {
                label: "เพิ่มได้เมื่ออยากตั้งเป้าหมายเอง".into(),
                tone: ReadinessTone::Optional,
            },
            "targets" => SectionReadiness {
                label: "มีเป้าหมายสำหรับเปรียบเทียบแล้ว".into(),
                tone: ReadinessTone::Ready,
            },
            "health"
                if self.health_scores.len() == 12
                    && self.health_scores.iter().all(|score| !score.is_empty()) =>
            {
                SectionReadiness {
                    label: "พร้อมดูแบบประเมินสวน".into(),
                    tone: ReadinessTone::Ready,
                }
            }
            "health" => SectionReadiness {
                label: "เพิ่มได้เพื่อทบทวนความพร้อมของสวน".into(),
                tone: ReadinessTone::Optional,
            },
            _ => SectionReadiness {
                label: "ยังไม่มีสถานะ".into(),
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
            "variable-costs" => {
                self.variable_costs = submitted.variable_costs.clone();
                self.variable_cost_state = submitted.variable_cost_state;
            }
            "fixed-costs" => {
                self.fixed_costs = submitted.fixed_costs.clone();
                self.fixed_cost_state = submitted.fixed_cost_state;
            }
            // Classifying moves an item across sections, so the capture page
            // owns all three cost lists together.
            "expenses" => {
                self.unclassified = submitted.unclassified.clone();
                self.variable_costs = submitted.variable_costs.clone();
                self.variable_cost_state = submitted.variable_cost_state;
                self.fixed_costs = submitted.fixed_costs.clone();
                self.fixed_cost_state = submitted.fixed_cost_state;
            }
            "tax-deductions" => self.tax_deductions = submitted.tax_deductions.clone(),
            "targets" => self.targets = submitted.targets.clone(),
            "health" => self.health_scores = submitted.health_scores.clone(),
            _ => return false,
        }
        true
    }

    /// File a remembered expense into the section the owner's answers point
    /// to, carrying its name and amount. Returns false when there is no such
    /// item.
    pub fn classify_expense(
        &mut self,
        index: usize,
        classification: ExpenseClassification,
    ) -> bool {
        if index >= self.unclassified.len() {
            return false;
        }
        let expense = self.unclassified.remove(index);
        match classification {
            ExpenseClassification::Variable(kind) => self.variable_costs.push(VariableCostForm {
                name: expense.name,
                kind,
                total_only: true,
                total_amount: expense.amount,
                ..VariableCostForm::default()
            }),
            ExpenseClassification::Fixed(cash_kind) => self.fixed_costs.push(FixedCostForm {
                name: expense.name,
                cash_kind,
                amount_per_year: expense.amount,
                investment_base: String::new(),
            }),
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
        "variable-costs" => field.starts_with("variable_costs[") || field == "variable_cost_state",
        "fixed-costs" => field.starts_with("fixed_costs[") || field == "fixed_cost_state",
        "expenses" => {
            field.starts_with("unclassified_expenses[")
                || field.starts_with("variable_costs[")
                || field.starts_with("fixed_costs[")
        }
        "tax-deductions" => field.starts_with("tax_deductions["),
        "targets" => field.starts_with("targets."),
        "health" => field.starts_with("health_answers"),
        _ => false,
    }
}

/// The hub row and the tax screen say the same thing about entered lines.
pub fn deductions_entered_label(plan: &Plan) -> String {
    format!(
        "กรอกแล้ว {} รายการ · รวม {} บาท",
        plan.tax_deductions.len(),
        crate::plan_ui::baht_amount(plan.tax_deduction_total())
    )
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
                label: "พอคำนวณรายได้แล้ว".into(),
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
                label: "ยังขาดราคาขายเฉลี่ย".into(),
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

    #[test]
    fn cost_readiness_names_unknown_confirmed_none_and_waiting_captures() {
        let mut form = PlanForm::from_plan(&calc::Plan::default());
        assert_eq!(
            form.section_readiness("variable-costs").label,
            "ยังขาดค่าใช้จ่ายตามการผลิต หรือยืนยันว่าไม่มี"
        );
        assert_eq!(
            form.section_readiness("expenses").tone,
            ReadinessTone::Optional
        );

        form.variable_cost_state = CostSectionState::ConfirmedNone;
        form.fixed_cost_state = CostSectionState::ConfirmedNone;
        assert_eq!(
            form.section_readiness("variable-costs").label,
            "ยืนยันแล้วว่าไม่มีค่าใช้จ่ายส่วนนี้"
        );
        assert_eq!(
            form.section_readiness("fixed-costs").tone,
            ReadinessTone::Ready
        );

        form.unclassified.push(UnclassifiedExpenseForm {
            name: "อะไรสักอย่าง".into(),
            amount: "100".into(),
            note: String::new(),
        });
        assert_eq!(
            form.section_readiness("expenses").label,
            "มี 1 รายการยังไม่ได้บอกว่าเป็นแบบไหน"
        );
        assert_eq!(
            form.section_readiness("expenses").tone,
            ReadinessTone::Missing
        );

        form.fixed_costs.push(FixedCostForm::default());
        form.fixed_costs[0].amount_per_year = "1000".into();
        let errors = form
            .to_plan()
            .expect_err("confirmed none with rows is refused");
        assert!(errors.iter().any(|error| error.field == "fixed_cost_state"));
        assert!(!form.section_errors("fixed-costs").is_empty());
    }

    #[test]
    fn the_capture_section_owns_all_three_cost_lists_and_others_stay_local() {
        let mut stored = PlanForm::from_plan(&calc::workbook_sample());
        let mut submitted = stored.clone();
        submitted
            .unclassified
            .push(UnclassifiedExpenseForm::default());
        submitted.classify_expense(0, ExpenseClassification::Fixed(CashKind::NonCash));
        submitted.market.target_customer = "ข้อมูลที่ไม่ควรมา".into();

        assert!(stored.replace_section_from("expenses", &submitted));
        assert_eq!(stored.fixed_costs.len(), submitted.fixed_costs.len());
        assert_ne!(stored.market.target_customer, "ข้อมูลที่ไม่ควรมา");

        let mut only_variable = PlanForm::from_plan(&calc::workbook_sample());
        let mut cleared = only_variable.clone();
        cleared.variable_costs.clear();
        cleared.variable_cost_state = CostSectionState::ConfirmedNone;
        cleared.fixed_costs.clear();
        assert!(only_variable.replace_section_from("variable-costs", &cleared));
        assert!(only_variable.variable_costs.is_empty());
        assert_eq!(
            only_variable.variable_cost_state,
            CostSectionState::ConfirmedNone
        );
        assert_eq!(
            only_variable.fixed_costs.len(),
            calc::workbook_sample().fixed_costs.len(),
            "fixed rows are untouched"
        );
    }
}
