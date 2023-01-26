-- Your SQL goes here

CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    filename TEXT NOT NULL,
    message TEXT NOT NULL
);
