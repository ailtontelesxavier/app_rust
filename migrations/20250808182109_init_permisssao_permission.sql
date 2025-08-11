-- migrations/..._create_permission_table.sql
CREATE TABLE IF NOT EXISTS permission (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    module_id INTEGER NOT NULL REFERENCES module(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Adiciona a constraint única apenas se não existir
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'uix_name_module'
    ) THEN
        ALTER TABLE permission ADD CONSTRAINT uix_name_module 
        UNIQUE (name, module_id);
    END IF;
END $$;


-- Tabela de módulos
CREATE TABLE module (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uk_module_title UNIQUE (title)
);

-- Índice para busca por título
CREATE INDEX idx_module_title ON module USING gin (title gin_trgm_ops);

-- Tabela de permissões
CREATE TABLE permission (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    module_id INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uk_permission_name_module UNIQUE (name, module_id),
    CONSTRAINT fk_permission_module FOREIGN KEY (module_id) 
        REFERENCES module(id) ON DELETE CASCADE
);

-- Índices para busca
CREATE INDEX idx_permission_name ON permission USING gin (name gin_trgm_ops);
CREATE INDEX idx_permission_description ON permission USING gin (description gin_trgm_ops);
CREATE INDEX idx_permission_module ON permission (module_id);

-- Função para atualizar o updated_at automaticamente
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers para atualização automática
CREATE TRIGGER update_module_timestamp
BEFORE UPDATE ON module
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_permission_timestamp
BEFORE UPDATE ON permission
FOR EACH ROW EXECUTE FUNCTION update_timestamp();