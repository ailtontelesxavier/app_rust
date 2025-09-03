-- Add migration script here
ALTER TABLE contato
ADD COLUMN dados_imports JSONB;