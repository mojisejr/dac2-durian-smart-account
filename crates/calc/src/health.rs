use rust_decimal::Decimal;

use crate::{
    HealthAnalysis, HealthDimension, HealthDimensionResult, HealthQuestion, HealthStatus, Plan,
};

const DIMENSIONS: [HealthDimension; 6] = [
    HealthDimension::Finance,
    HealthDimension::Production,
    HealthDimension::Market,
    HealthDimension::Resources,
    HealthDimension::People,
    HealthDimension::Resilience,
];

pub fn calculate(plan: &Plan) -> HealthAnalysis {
    let dimensions = DIMENSIONS
        .into_iter()
        .map(|dimension| {
            let scores: Vec<_> = HealthQuestion::ALL
                .into_iter()
                .filter(|question| question.dimension() == dimension)
                .map(|question| score_for(plan, question))
                .collect();
            let average = average_complete(&scores);
            HealthDimensionResult {
                dimension,
                average,
                status: average.map(status),
            }
        })
        .collect();

    let all_scores: Vec<_> = HealthQuestion::ALL
        .into_iter()
        .map(|question| score_for(plan, question))
        .collect();
    let overall_score = average_complete(&all_scores);

    HealthAnalysis {
        dimensions,
        overall_score,
        overall_status: overall_score.map(status),
    }
}

fn score_for(plan: &Plan, question: HealthQuestion) -> Option<Decimal> {
    let mut answers = plan
        .health_answers
        .iter()
        .filter(|answer| answer.question == question);
    let answer = answers.next()?;
    if answers.next().is_some() {
        return None;
    }
    answer
        .score
        .filter(|score| (1..=5).contains(score))
        .map(Decimal::from)
}

fn average_complete(scores: &[Option<Decimal>]) -> Option<Decimal> {
    let total = scores
        .iter()
        .copied()
        .try_fold(Decimal::ZERO, |total, score| {
            score.map(|score| total + score)
        })?;
    (!scores.is_empty()).then(|| total / Decimal::from(scores.len()))
}

fn status(score: Decimal) -> HealthStatus {
    if score >= Decimal::from(4) {
        HealthStatus::Strong
    } else if score >= Decimal::from(3) {
        HealthStatus::Watch
    } else {
        HealthStatus::ImproveUrgently
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workbook_sample;

    #[test]
    fn workbook_scores_all_six_dimensions_at_three() {
        let result = calculate(&workbook_sample());
        assert_eq!(result.dimensions.len(), 6);
        assert!(
            result
                .dimensions
                .iter()
                .all(|item| item.average == Some(Decimal::from(3)))
        );
        assert_eq!(result.overall_score, Some(Decimal::from(3)));
        assert_eq!(result.overall_status, Some(HealthStatus::Watch));
    }

    #[test]
    fn lowest_and_highest_scores_map_to_the_expected_status() {
        let mut plan = workbook_sample();
        for answer in &mut plan.health_answers {
            answer.score = Some(1);
        }
        assert_eq!(
            calculate(&plan).overall_status,
            Some(HealthStatus::ImproveUrgently)
        );

        for answer in &mut plan.health_answers {
            answer.score = Some(5);
        }
        assert_eq!(calculate(&plan).overall_status, Some(HealthStatus::Strong));
    }

    #[test]
    fn missing_invalid_or_duplicate_answers_do_not_produce_a_score() {
        let mut plan = workbook_sample();
        plan.health_answers[0].score = None;
        assert_eq!(calculate(&plan).overall_score, None);

        plan = workbook_sample();
        plan.health_answers[0].score = Some(6);
        assert_eq!(calculate(&plan).overall_score, None);

        plan = workbook_sample();
        plan.health_answers.push(plan.health_answers[0].clone());
        assert_eq!(calculate(&plan).overall_score, None);
    }
}
