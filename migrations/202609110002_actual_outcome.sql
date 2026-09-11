CREATE TABLE season_actual_outcomes (
    plan_id BIGINT PRIMARY KEY,
    owner_id BIGINT NOT NULL,
    sellable_yield_kg NUMERIC,
    revenue NUMERIC,
    total_cost NUMERIC,
    note TEXT NOT NULL DEFAULT '',
    finalized_at TIMESTAMPTZ,
    forecast_mode TEXT,
    forecast_sellable_yield_kg NUMERIC,
    forecast_revenue NUMERIC,
    forecast_total_cost NUMERIC,
    forecast_profit NUMERIC,
    forecast_average_price_per_kg NUMERIC,
    forecast_cost_per_kg NUMERIC,
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE,
    CONSTRAINT season_actual_yield_nonnegative
        CHECK (sellable_yield_kg IS NULL OR sellable_yield_kg >= 0),
    CONSTRAINT season_actual_revenue_nonnegative
        CHECK (revenue IS NULL OR revenue >= 0),
    CONSTRAINT season_actual_cost_nonnegative
        CHECK (total_cost IS NULL OR total_cost >= 0),
    CONSTRAINT season_actual_note_length
        CHECK (char_length(note) <= 2000),
    CONSTRAINT season_actual_forecast_mode_valid
        CHECK (forecast_mode IS NULL OR forecast_mode IN ('quick', 'detailed')),
    CONSTRAINT season_actual_finalized_shape
        CHECK (
            (finalized_at IS NULL AND forecast_mode IS NULL)
            OR
            (
                finalized_at IS NOT NULL
                AND forecast_mode IS NOT NULL
                AND sellable_yield_kg IS NOT NULL
                AND revenue IS NOT NULL
                AND total_cost IS NOT NULL
            )
        )
);
