use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{HealthQuestion, Plan};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum InputIssueKind {
    Negative,
    OutsideShareRange,
    GradeSharesDoNotTotalOne,
    HealthScoreOutsideRange,
    DuplicateHealthAnswer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InputIssue {
    pub field: String,
    pub kind: InputIssueKind,
}

impl Plan {
    pub fn input_issues(&self) -> Vec<InputIssue> {
        let mut issues = Vec::new();

        check_non_negative(&mut issues, "market.demand_kg", self.market.demand_kg);
        check_non_negative(
            &mut issues,
            "market.minimum_price_per_kg",
            self.market.minimum_price_per_kg,
        );
        check_share(
            &mut issues,
            "market.largest_buyer_share",
            self.market.largest_buyer_share,
        );
        check_non_negative(&mut issues, "production.area_rai", self.production.area_rai);
        check_non_negative(
            &mut issues,
            "production.producing_trees",
            self.production.producing_trees,
        );
        check_non_negative(
            &mut issues,
            "production.fruits_per_tree",
            self.production.fruits_per_tree,
        );
        check_non_negative(
            &mut issues,
            "production.average_fruit_weight_kg",
            self.production.average_fruit_weight_kg,
        );
        check_share(
            &mut issues,
            "production.loss_share",
            self.production.loss_share,
        );

        let mut grade_total = Decimal::ZERO;
        let mut all_grade_shares_present = !self.production.grades.is_empty();
        for (index, grade) in self.production.grades.iter().enumerate() {
            check_share(
                &mut issues,
                &format!("production.grades[{index}].share"),
                grade.share,
            );
            check_non_negative(
                &mut issues,
                &format!("production.grades[{index}].price_per_kg"),
                grade.price_per_kg,
            );
            match grade.share {
                Some(share) => grade_total += share,
                None => all_grade_shares_present = false,
            }
        }
        if all_grade_shares_present && (grade_total - Decimal::ONE).abs() >= Decimal::new(1, 4) {
            issues.push(InputIssue {
                field: "production.grades".into(),
                kind: InputIssueKind::GradeSharesDoNotTotalOne,
            });
        }

        for (index, line) in self.variable_costs.iter().enumerate() {
            check_non_negative(
                &mut issues,
                &format!("variable_costs[{index}].quantity"),
                line.quantity,
            );
            check_non_negative(
                &mut issues,
                &format!("variable_costs[{index}].unit_price"),
                line.unit_price,
            );
        }
        for (index, line) in self.fixed_costs.iter().enumerate() {
            check_non_negative(
                &mut issues,
                &format!("fixed_costs[{index}].amount_per_year"),
                line.amount_per_year,
            );
            check_non_negative(
                &mut issues,
                &format!("fixed_costs[{index}].investment_base"),
                line.investment_base,
            );
        }

        for question in HealthQuestion::ALL {
            let answers: Vec<_> = self
                .health_answers
                .iter()
                .filter(|answer| answer.question == question)
                .collect();
            if answers.len() > 1 {
                issues.push(InputIssue {
                    field: format!("health_answers.{question:?}"),
                    kind: InputIssueKind::DuplicateHealthAnswer,
                });
            }
            for answer in answers {
                if answer.score.is_some_and(|score| !(1..=5).contains(&score)) {
                    issues.push(InputIssue {
                        field: format!("health_answers.{question:?}"),
                        kind: InputIssueKind::HealthScoreOutsideRange,
                    });
                }
            }
        }

        for (kind, value) in self.targets.values() {
            check_non_negative(&mut issues, &format!("targets.{kind:?}"), value);
        }
        check_share(
            &mut issues,
            "targets.quality_grade_share",
            self.targets.quality_grade_share,
        );
        check_share(&mut issues, "targets.loss_share", self.targets.loss_share);

        issues
    }
}

fn check_non_negative(issues: &mut Vec<InputIssue>, field: &str, value: Option<Decimal>) {
    if value.is_some_and(|value| value.is_sign_negative()) {
        issues.push(InputIssue {
            field: field.into(),
            kind: InputIssueKind::Negative,
        });
    }
}

fn check_share(issues: &mut Vec<InputIssue>, field: &str, value: Option<Decimal>) {
    if value.is_some_and(|value| !(Decimal::ZERO..=Decimal::ONE).contains(&value)) {
        issues.push(InputIssue {
            field: field.into(),
            kind: InputIssueKind::OutsideShareRange,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workbook_sample;

    #[test]
    fn workbook_sample_has_no_input_contract_issues() {
        assert!(workbook_sample().input_issues().is_empty());
    }

    #[test]
    fn invalid_money_shares_and_scores_are_reported() {
        let mut plan = workbook_sample();
        plan.production.area_rai = Some(Decimal::NEGATIVE_ONE);
        plan.production.loss_share = Some(Decimal::new(11, 1));
        plan.health_answers[0].score = Some(6);

        let issues = plan.input_issues();
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == InputIssueKind::Negative)
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == InputIssueKind::OutsideShareRange)
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == InputIssueKind::HealthScoreOutsideRange)
        );
    }

    #[test]
    fn grade_total_and_duplicate_health_answer_are_reported() {
        let mut plan = workbook_sample();
        plan.production.grades[0].share = Some(Decimal::new(49, 2));
        plan.health_answers.push(plan.health_answers[0].clone());

        let issues = plan.input_issues();
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == InputIssueKind::GradeSharesDoNotTotalOne)
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == InputIssueKind::DuplicateHealthAnswer)
        );
    }
}
