ALTER TABLE plans
    ADD COLUMN forecast_mode TEXT NOT NULL DEFAULT 'detailed',
    ADD COLUMN quick_sellable_yield_kg NUMERIC,
    ADD COLUMN quick_average_price_per_kg NUMERIC,
    ADD COLUMN quick_total_cost NUMERIC,
    ADD CONSTRAINT plans_forecast_mode_valid
        CHECK (forecast_mode IN ('quick', 'detailed')),
    ADD CONSTRAINT plans_quick_sellable_yield_positive
        CHECK (quick_sellable_yield_kg IS NULL OR quick_sellable_yield_kg > 0),
    ADD CONSTRAINT plans_quick_average_price_positive
        CHECK (quick_average_price_per_kg IS NULL OR quick_average_price_per_kg > 0),
    ADD CONSTRAINT plans_quick_total_cost_positive
        CHECK (quick_total_cost IS NULL OR quick_total_cost > 0);
