ALTER TABLE plans
    ADD COLUMN starting_capital NUMERIC,
    ADD CONSTRAINT plans_starting_capital_positive
        CHECK (starting_capital IS NULL OR (starting_capital > 0 AND starting_capital <= 1000000000000));

CREATE TABLE owner_assets (
    id BIGSERIAL PRIMARY KEY,
    owner_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (char_length(BTRIM(name)) BETWEEN 1 AND 120),
    kind TEXT NOT NULL CHECK (kind IN ('equipment', 'owned_land')),
    original_cost NUMERIC NOT NULL CHECK (original_cost > 0 AND original_cost <= 1000000000000),
    start_year INTEGER NOT NULL CHECK (start_year BETWEEN 1000 AND 9999),
    useful_life_years INTEGER,
    residual_value NUMERIC,
    retired_year INTEGER CHECK (retired_year BETWEEN 1000 AND 9999),
    UNIQUE (id, owner_id),
    CONSTRAINT owner_assets_retirement_after_start
        CHECK (retired_year IS NULL OR retired_year >= start_year),
    CONSTRAINT owner_assets_kind_shape
        CHECK (
            (
                kind = 'equipment'
                AND useful_life_years BETWEEN 1 AND 200
                AND (residual_value IS NULL OR (residual_value >= 0 AND residual_value <= original_cost))
            )
            OR
            (kind = 'owned_land' AND useful_life_years IS NULL AND residual_value IS NULL)
        )
);

CREATE INDEX owner_assets_owner_id_idx ON owner_assets(owner_id, id);

CREATE TABLE season_asset_selections (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    asset_id BIGINT NOT NULL,
    PRIMARY KEY (plan_id, asset_id),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id, owner_id) REFERENCES owner_assets(id, owner_id) ON DELETE CASCADE
);

CREATE TABLE season_asset_snapshots (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    source_asset_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('equipment', 'owned_land')),
    original_cost NUMERIC NOT NULL,
    start_year INTEGER NOT NULL,
    useful_life_years INTEGER,
    residual_value NUMERIC,
    retired_year INTEGER,
    annual_depreciation NUMERIC NOT NULL CHECK (annual_depreciation >= 0),
    residual_assumed_zero BOOLEAN NOT NULL,
    PRIMARY KEY (plan_id, position),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);

CREATE INDEX season_asset_snapshots_owner_plan_idx
    ON season_asset_snapshots(owner_id, plan_id, position);
