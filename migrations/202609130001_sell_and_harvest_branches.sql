-- Batch 2: sell and harvest in familiar branches.
--
-- The stored market quantity was only ever used as "what a buyer said they
-- would take" (market gap and fulfillment arithmetic). The column name now
-- says so. Values are unchanged.
ALTER TABLE market_plans RENAME COLUMN demand_kg TO buyer_committed_kg;

-- A detailed plan now records which production and price branch the owner
-- chose. Existing rows keep their previous meaning: yield derived from orchard
-- facts and price weighted by grade. The unselected branch's facts are kept.
ALTER TABLE yield_estimates
    ADD COLUMN yield_source TEXT NOT NULL DEFAULT 'derived',
    ADD COLUMN sellable_yield_kg NUMERIC,
    ADD COLUMN price_source TEXT NOT NULL DEFAULT 'by_grade',
    ADD COLUMN average_price_per_kg NUMERIC,
    ADD CONSTRAINT yield_estimates_yield_source_valid
        CHECK (yield_source IN ('direct', 'derived')),
    ADD CONSTRAINT yield_estimates_sellable_yield_nonnegative
        CHECK (sellable_yield_kg IS NULL OR sellable_yield_kg >= 0),
    ADD CONSTRAINT yield_estimates_price_source_valid
        CHECK (price_source IN ('average', 'by_grade')),
    ADD CONSTRAINT yield_estimates_average_price_nonnegative
        CHECK (average_price_per_kg IS NULL OR average_price_per_kg >= 0);
