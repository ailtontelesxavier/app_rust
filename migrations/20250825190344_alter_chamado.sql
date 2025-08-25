-- Add migration script here
ALTER TABLE chamado_chamados
    ALTER COLUMN created_at SET DEFAULT NOW(),
    ALTER COLUMN updated_at SET DEFAULT NOW(),
    ALTER COLUMN status SET DEFAULT 0;

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
   NEW.updated_at = NOW();
   RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER set_updated_at
BEFORE UPDATE ON chamado_chamados
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE chamado_chamados
    ADD CONSTRAINT chk_status_chamado CHECK (status IN (0,1,2,3,4));
