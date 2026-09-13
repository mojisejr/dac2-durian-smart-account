-- Rollback for 202609130001_sell_and_harvest_branches.sql.
--
-- sqlx runs only the top-level migrations directory; this file is applied by
-- the store test that proves existing rows survive both directions, and by an
-- operator who needs to step the schema back by hand.
ALTER TABLE yield_estimates
    DROP CONSTRAINT yield_estimates_average_price_nonnegative,
    DROP CONSTRAINT yield_estimates_price_source_valid,
    DROP CONSTRAINT yield_estimates_sellable_yield_nonnegative,
    DROP CONSTRAINT yield_estimates_yield_source_valid,
    DROP COLUMN average_price_per_kg,
    DROP COLUMN price_source,
    DROP COLUMN sellable_yield_kg,
    DROP COLUMN yield_source;

ALTER TABLE market_plans RENAME COLUMN buyer_committed_kg TO demand_kg;

DELETE FROM _sqlx_migrations WHERE version = 202609130001;
