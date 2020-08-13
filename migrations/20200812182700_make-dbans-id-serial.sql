-- Add migration script here
ALTER TABLE dbans DROP COLUMN id;
ALTER TABLE dbans ADD COLUMN id SERIAL PRIMARY KEY;
