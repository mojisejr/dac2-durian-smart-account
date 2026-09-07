CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE
);

CREATE TABLE plans (
    id BIGSERIAL PRIMARY KEY,
    owner_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    closed_at TIMESTAMPTZ,
    UNIQUE (id, owner_id)
);

CREATE INDEX plans_owner_id_idx ON plans(owner_id);

CREATE TABLE market_plans (
    plan_id BIGINT PRIMARY KEY,
    owner_id BIGINT NOT NULL,
    target_customer TEXT,
    demand_kg NUMERIC,
    minimum_price_per_kg NUMERIC,
    sales_period TEXT,
    sales_channels BIGINT,
    largest_buyer_share NUMERIC,
    quality_requirements TEXT,
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);

CREATE TABLE yield_estimates (
    plan_id BIGINT PRIMARY KEY,
    owner_id BIGINT NOT NULL,
    area_rai NUMERIC,
    producing_trees NUMERIC,
    fruits_per_tree NUMERIC,
    average_fruit_weight_kg NUMERIC,
    loss_share NUMERIC,
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);

CREATE TABLE grade_mix (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    name TEXT NOT NULL,
    share NUMERIC,
    price_per_kg NUMERIC,
    counts_as_quality_grade BOOLEAN NOT NULL,
    PRIMARY KEY (plan_id, position),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);

CREATE TABLE variable_cost_lines (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN (
        'fertilizer', 'crop_protection', 'water', 'orchard_labor',
        'electricity', 'fuel', 'harvest_labor', 'transport', 'packing',
        'maintenance', 'other'
    )),
    quantity NUMERIC,
    unit TEXT NOT NULL,
    unit_price NUMERIC,
    PRIMARY KEY (plan_id, position),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);

CREATE TABLE fixed_cost_lines (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    name TEXT NOT NULL,
    cash_kind TEXT NOT NULL CHECK (cash_kind IN ('cash', 'non_cash')),
    amount_per_year NUMERIC,
    investment_base NUMERIC,
    PRIMARY KEY (plan_id, position),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);

CREATE TABLE health_answers (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    question TEXT NOT NULL CHECK (question IN (
        'profit_and_cash', 'next_season_reserve', 'yield_and_quality',
        'loss_control', 'multiple_sales_channels', 'price_volatility',
        'resource_efficiency', 'environmental_care', 'fair_and_safe_work',
        'labor_continuity', 'downside_survival', 'contingency_plan'
    )),
    score SMALLINT,
    PRIMARY KEY (plan_id, position),
    UNIQUE (plan_id, question),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);

CREATE TABLE kpi_targets (
    plan_id BIGINT PRIMARY KEY,
    owner_id BIGINT NOT NULL,
    yield_per_rai NUMERIC,
    yield_per_tree NUMERIC,
    yield_per_labor_day NUMERIC,
    yield_per_fertilizer_kg NUMERIC,
    yield_per_water_cubic_meter NUMERIC,
    yield_per_kwh NUMERIC,
    quality_grade_share NUMERIC,
    loss_share NUMERIC,
    cost_per_kg NUMERIC,
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);
