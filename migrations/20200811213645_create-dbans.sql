-- Add migration script here
CREATE TABLE dbans (
  id INTEGER PRIMARY KEY,
  userid TEXT NOT NULL,
  reason TEXT NOT NULL,
  guild_id TEXT NOT NULL,
  is_withdrawn BOOLEAN NOT NULL DEFAULT 'f'
);
