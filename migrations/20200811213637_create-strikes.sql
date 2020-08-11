-- Add migration script here
CREATE TABLE strikes (
  id INTEGER PRIMARY KEY,
  userid TEXT NOT NULL,
  moderator TEXT NOT NULL,
  reason TEXT NOT NULL,
  details TEXT,
  is_withdrawn BOOLEAN NOT NULL DEFAULT 'f'
);
