use rust_decimal::Decimal;

use crate::{
    CashKind, FixedCostLine, Grade, HealthAnswer, HealthQuestion, KpiTargets, MarketPlan, Plan,
    ProductionPlan, VariableCostKind, VariableCostLine,
};

pub const WORKBOOK_SHA256: &str =
    "49296f67fae536733593350f3ee493811e9231763229ff5c4faded319cbb479e";

pub fn workbook_sample() -> Plan {
    Plan {
        name: "ตัวอย่างจากแบบคำนวณ".into(),
        market: MarketPlan {
            target_customer: Some("ล้งส่งออก".into()),
            demand_kg: Some(Decimal::from(25_000)),
            minimum_price_per_kg: Some(Decimal::from(60)),
            sales_period: Some("พฤษภาคม–มิถุนายน".into()),
            sales_channels: Some(2),
            largest_buyer_share: Some(Decimal::new(7, 1)),
            quality_requirements: Some("น้ำหนัก/ความสุก/เกรด".into()),
        },
        production: ProductionPlan {
            area_rai: Some(Decimal::from(10)),
            producing_trees: Some(Decimal::from(200)),
            fruits_per_tree: Some(Decimal::from(35)),
            average_fruit_weight_kg: Some(Decimal::from(3)),
            loss_share: Some(Decimal::new(5, 2)),
            grades: vec![
                grade("A", 5, 1, 100, true),
                grade("B", 3, 1, 80, true),
                grade("C", 15, 2, 50, false),
                grade("ตกเกรด", 5, 2, 20, false),
            ],
        },
        variable_costs: vec![
            variable(
                "ปุ๋ยและธาตุอาหาร",
                VariableCostKind::Fertilizer,
                Some(5_000),
                "กก.",
                200,
                1,
            ),
            variable(
                "สารเคมี/ชีวภัณฑ์",
                VariableCostKind::CropProtection,
                Some(50),
                "ชุด",
                2_000,
                0,
            ),
            variable("น้ำ", VariableCostKind::Water, Some(2_500), "ลบ.ม.", 8, 0),
            variable(
                "แรงงานดูแลสวน",
                VariableCostKind::OrchardLabor,
                Some(100),
                "วัน",
                600,
                0,
            ),
            variable(
                "ไฟฟ้า",
                VariableCostKind::Electricity,
                Some(12_000),
                "kWh",
                42,
                1,
            ),
            variable("น้ำมัน", VariableCostKind::Fuel, Some(1_200), "ลิตร", 30, 0),
            variable(
                "แรงงานเก็บเกี่ยว",
                VariableCostKind::HarvestLabor,
                None,
                "กก.",
                3,
                0,
            ),
            variable("ขนส่ง", VariableCostKind::Transport, None, "กก.", 1, 0),
            variable(
                "คัดแยก/บรรจุภัณฑ์",
                VariableCostKind::Packing,
                None,
                "กก.",
                5,
                1,
            ),
            variable(
                "ซ่อมบำรุง",
                VariableCostKind::Maintenance,
                Some(1),
                "ปี",
                100_000,
                0,
            ),
            variable("อื่นๆ 1", VariableCostKind::Other, Some(1), "ปี", 0, 0),
            variable("อื่นๆ 2", VariableCostKind::Other, Some(1), "ปี", 0, 0),
            variable("อื่นๆ 3", VariableCostKind::Other, Some(1), "ปี", 0, 0),
            variable("อื่นๆ 4", VariableCostKind::Other, Some(1), "ปี", 0, 0),
            variable("อื่นๆ 5", VariableCostKind::Other, Some(1), "ปี", 0, 0),
            variable("อื่นๆ 6", VariableCostKind::Other, Some(1), "ปี", 0, 0),
        ],
        fixed_costs: vec![
            fixed("ค่าเช่าที่ดิน", CashKind::Cash, 100_000, 0),
            fixed("ค่าเสื่อมระบบน้ำ", CashKind::NonCash, 18_000, 200_000),
            fixed("ค่าเสื่อมรถและเครื่องจักร", CashKind::NonCash, 45_100, 500_000),
            fixed("ดอกเบี้ยเงินกู้", CashKind::Cash, 20_000, 0),
            fixed("ค่าแรงประจำ", CashKind::Cash, 60_000, 0),
            fixed("บริหาร/บัญชี/โทรศัพท์", CashKind::Cash, 12_000, 0),
            fixed("ภาษีและค่าธรรมเนียม", CashKind::Cash, 0, 0),
            fixed("อื่นๆ 1", CashKind::Cash, 0, 0),
            fixed("อื่นๆ 2", CashKind::Cash, 0, 0),
            fixed("อื่นๆ 3", CashKind::Cash, 0, 0),
            fixed("อื่นๆ 4", CashKind::Cash, 0, 0),
        ],
        health_answers: HealthQuestion::ALL
            .into_iter()
            .map(|question| HealthAnswer {
                question,
                score: Some(3),
            })
            .collect(),
        targets: KpiTargets {
            yield_per_rai: Some(Decimal::from(2_200)),
            yield_per_tree: Some(Decimal::from(110)),
            yield_per_labor_day: Some(Decimal::from(180)),
            yield_per_fertilizer_kg: Some(Decimal::new(45, 1)),
            yield_per_water_cubic_meter: Some(Decimal::from(8)),
            yield_per_kwh: Some(Decimal::new(18, 1)),
            quality_grade_share: Some(Decimal::new(8, 1)),
            loss_share: Some(Decimal::new(5, 2)),
            cost_per_kg: Some(Decimal::from(45)),
        },
    }
}

fn grade(
    name: &str,
    share_mantissa: i64,
    share_scale: u32,
    price: i64,
    counts_as_quality_grade: bool,
) -> Grade {
    Grade {
        name: name.into(),
        share: Some(Decimal::new(share_mantissa, share_scale)),
        price_per_kg: Some(Decimal::from(price)),
        counts_as_quality_grade,
    }
}

fn variable(
    name: &str,
    kind: VariableCostKind,
    quantity: Option<i64>,
    unit: &str,
    price_mantissa: i64,
    price_scale: u32,
) -> VariableCostLine {
    VariableCostLine {
        name: name.into(),
        kind,
        quantity: quantity.map(Decimal::from),
        unit: unit.into(),
        unit_price: Some(Decimal::new(price_mantissa, price_scale)),
    }
}

fn fixed(
    name: &str,
    cash_kind: CashKind,
    amount_per_year: i64,
    investment_base: i64,
) -> FixedCostLine {
    FixedCostLine {
        name: name.into(),
        cash_kind,
        amount_per_year: Some(Decimal::from(amount_per_year)),
        investment_base: Some(Decimal::from(investment_base)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_preserves_all_owner_input_rows() {
        let plan = workbook_sample();
        assert_eq!(plan.production.grades.len(), 4);
        assert_eq!(plan.variable_costs.len(), 16);
        assert_eq!(plan.fixed_costs.len(), 11);
        assert_eq!(plan.health_answers.len(), 12);
        assert_eq!(
            plan.targets
                .values()
                .filter(|(_, value)| value.is_some())
                .count(),
            9
        );
    }
}
