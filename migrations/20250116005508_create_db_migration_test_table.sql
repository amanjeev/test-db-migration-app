CREATE TABLE IF NOT EXISTS dbmigrationtest
(
    -- Primary key
    id SERIAL PRIMARY KEY UNIQUE,

    -- Name
    name VARCHAR(255) NOT NULL,

    -- Inner thoughts
    thoughts VARCHAR(255) NOT NULL
);