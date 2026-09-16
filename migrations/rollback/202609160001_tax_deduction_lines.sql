-- Rollback for 202609160001_tax_deduction_lines.sql. Applied by the store
-- test that proves existing rows survive both directions, or by hand.
DROP TABLE tax_deduction_lines;

DELETE FROM _sqlx_migrations WHERE version = 202609160001;
