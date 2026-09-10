ALTER TABLE plans
    ADD COLUMN season_year INTEGER,
    ADD COLUMN note TEXT NOT NULL DEFAULT '',
    ADD CONSTRAINT plans_season_year_four_digits
        CHECK (season_year IS NULL OR season_year BETWEEN 1000 AND 9999);

CREATE UNIQUE INDEX plans_owner_season_year_unique
    ON plans(owner_id, season_year)
    WHERE season_year IS NOT NULL;
