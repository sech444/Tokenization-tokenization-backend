-- Add migration script here

-- migrations/009_add_users_active.sql
ALTER TABLE users ADD COLUMN active BOOLEAN DEFAULT true;