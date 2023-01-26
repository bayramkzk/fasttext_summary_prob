-- Your SQL goes here

CREATE TABLE summary_probs (
    id SERIAL PRIMARY KEY,
    message_id INTEGER NOT NULL,
    lang TEXT NOT NULL,
    prob REAL NOT NULL,

    FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE
);
