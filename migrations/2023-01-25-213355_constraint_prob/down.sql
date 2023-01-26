-- This file should undo anything in `up.sql`

ALTER TABLE summary_probs DROP CONSTRAINT summary_probs_unique;
