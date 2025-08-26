-- Add migration script here
ALTER TABLE chamado_chamados
    ALTER COLUMN descricao TYPE JSONB USING descricao::JSONB;
