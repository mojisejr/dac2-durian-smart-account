-- Tax deductions the owner enters one line at a time, per season.
--
-- The estimate used to subtract a constant 60,000 that no screen could
-- change. Now the owner's own lines are summed; a season with no lines
-- deducts nothing and the screen says so. There is no "confirmed none" for
-- deductions, so no state column: rows present means entered.
CREATE TABLE tax_deduction_lines (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    amount NUMERIC NOT NULL CHECK (amount >= 0),
    PRIMARY KEY (plan_id, position),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE
);
