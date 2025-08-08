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