-- Your SQL goes here

ALTER TABLE summary_probs ADD CONSTRAINT summary_probs_unique UNIQUE (message_id, lang);
