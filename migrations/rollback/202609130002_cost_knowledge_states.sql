-- Rollback for 202609130002_cost_knowledge_states.sql. Applied by the store
-- test that proves existing rows survive both directions, or by hand.
ALTER TABLE variable_cost_lines
    DROP CONSTRAINT variable_cost_lines_total_or_unit,
    DROP CONSTRAINT variable_cost_lines_total_amount_nonnegative,
    DROP COLUMN total_amount;

DROP TABLE unclassified_expenses;

ALTER TABLE plans
    DROP CONSTRAINT plans_fixed_cost_state_valid,
    DROP CONSTRAINT plans_variable_cost_state_valid,
    DROP COLUMN fixed_cost_state,
    DROP COLUMN variable_cost_state;

DELETE FROM _sqlx_migrations WHERE version = 202609130002;
