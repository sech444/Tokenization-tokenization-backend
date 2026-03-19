-- Add migration script here

-- migrations/008_add_tokens_metadata.sql
ALTER TABLE tokens ADD COLUMN metadata JSONB;