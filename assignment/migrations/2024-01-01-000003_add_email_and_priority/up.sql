ALTER TABLE users ADD COLUMN email VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE tasks ADD COLUMN priority VARCHAR(50) NOT NULL DEFAULT 'medium';

UPDATE users SET email = 'admin@example.com' WHERE username = 'admin';
UPDATE users SET email = 'james_bond@example.com' WHERE username = 'james_bond';
