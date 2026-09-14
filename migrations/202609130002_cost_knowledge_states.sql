-- Batch 3: spend, own, and invest without classifying first.
--
-- An empty cost list used to be indistinguishable from an unknown one. Each
-- cost section now carries an explicit knowledge state. Existing rows default
-- to unknown, which is what an empty list already meant.
ALTER TABLE plans
    ADD COLUMN variable_cost_state TEXT NOT NULL DEFAULT 'unknown',
    ADD COLUMN fixed_cost_state TEXT NOT NULL DEFAULT 'unknown',
    ADD CONSTRAINT plans_variable_cost_state_valid
        CHECK (variable_cost_state IN ('unknown', 'confirmed_none', 'entered_items')),
    ADD CONSTRAINT plans_fixed_cost_state_valid
        CHECK (fixed_cost_state IN ('unknown', 'confirmed_none', 'entered_items'));

-- A remembered expense the owner has not classified yet. It enters no
-- calculation; classifying it moves it into a cost section and deletes it.
CREATE TABLE unclassified_expenses (
    plan_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    name TEXT NOT NULL,
    amount NUMERIC,
    note TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (plan_id, position),
    FOREIGN KEY (plan_id, owner_id) REFERENCES plans(id, owner_id) ON DELETE CASCADE,
    CONSTRAINT unclassified_expenses_amount_nonnegative
        CHECK (amount IS NULL OR amount >= 0),
    CONSTRAINT unclassified_expenses_note_length
        CHECK (char_length(note) <= 2000)
);

-- A variable line may carry a total instead of quantity times unit price.
-- Never both: the application reports that as an input issue, and the
-- database refuses it so nothing can be resolved silently.
ALTER TABLE variable_cost_lines
    ADD COLUMN total_amount NUMERIC,
    ADD CONSTRAINT variable_cost_lines_total_amount_nonnegative
        CHECK (total_amount IS NULL OR total_amount >= 0),
    ADD CONSTRAINT variable_cost_lines_total_or_unit
        CHECK (total_amount IS NULL OR (quantity IS NULL AND unit_price IS NULL));
