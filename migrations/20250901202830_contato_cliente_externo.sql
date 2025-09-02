-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- linha
CREATE TABLE linha (
    id SERIAL PRIMARY KEY,
    nome VARCHAR(255) NOT NULL,
    permite_cnpj BOOLEAN NOT NULL DEFAULT false,
    permite_cpf BOOLEAN NOT NULL DEFAULT false,
    permite_avalista BOOLEAN NOT NULL DEFAULT false,
    valor_maximo NUMERIC(18,2) NOT NULL
);

-- contato
CREATE TABLE contato (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    linha_id INT NOT NULL REFERENCES linha(id),
    protocolo VARCHAR(100) NOT NULL,
    status_atendimento BOOLEAN DEFAULT false,
    cpf_cnpj VARCHAR(14) NOT NULL,
    nome VARCHAR(255) NOT NULL,
    telefone VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL,
    cidade_id BIGINT NOT NULL REFERENCES municipio(id),
    val_solicitado NUMERIC(18,2) NOT NULL,
    status_tramitacao INT NOT NULL,
    campos JSONB NOT NULL, -- armazena Campos
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    CONSTRAINT email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- aplicacao_recurso
CREATE TABLE aplicacao_recurso (
    id BIGSERIAL PRIMARY KEY,
    descricao TEXT NOT NULL,
    quantidade INT NOT NULL,
    valor_unitario NUMERIC(18,2) NOT NULL,
    valor_total NUMERIC(18,2) NOT NULL,
    contato_id UUID NOT NULL REFERENCES contato(id) ON DELETE CASCADE
);

-- doc_solicitante
CREATE TABLE doc_solicitante (
    id BIGSERIAL PRIMARY KEY,
    contato_id UUID NOT NULL REFERENCES contato(id) ON DELETE CASCADE,
    arquivo TEXT NOT NULL, -- caminho ou hash do arquivo
    status_arquivo VARCHAR(50) NOT NULL,
    observacao TEXT,
    tipo VARCHAR(100) NOT NULL
);

-- tipo_documento
CREATE TABLE tipo_documento (
    id SERIAL PRIMARY KEY,
    nome VARCHAR(255) NOT NULL,
    descricao TEXT
);

-- tipo_doc_contato
CREATE TABLE tipo_doc_contato (
    id SERIAL PRIMARY KEY,
    tipo_documento_id INT NOT NULL REFERENCES tipo_documento(id), -- ON DELETE CASCADE,
    contato_id UUID NOT NULL REFERENCES contato(id) ON DELETE CASCADE,
    obrigatorio BOOLEAN NOT NULL DEFAULT false,
    ativo BOOLEAN NOT NULL DEFAULT true
);

-- contato_socio
CREATE TABLE contato_socio (
    id BIGSERIAL PRIMARY KEY,
    contato_id UUID NOT NULL REFERENCES contato(id) ON DELETE CASCADE,
    nome VARCHAR(255) NOT NULL,
    cpf VARCHAR(14) NOT NULL,
    telefone VARCHAR(50),
    email VARCHAR(255),
    exporta_politicamente BOOLEAN NOT NULL DEFAULT false,
    nome_conj VARCHAR(255),
    cpf_conj VARCHAR(14),
    telefone_conj VARCHAR(50),
    email_conj VARCHAR(255)
);

--contato_avalista
CREATE TABLE contato_avalista (
    id BIGSERIAL PRIMARY KEY,
    contato_id UUID NOT NULL REFERENCES contato(id) ON DELETE CASCADE,
    nome VARCHAR(255) NOT NULL,
    cpf VARCHAR(14) NOT NULL,
    telefone VARCHAR(50),
    email VARCHAR(255),
    exporta_politicamente BOOLEAN NOT NULL DEFAULT false,
    nome_conj VARCHAR(255),
    cpf_conj VARCHAR(14),
    telefone_conj VARCHAR(50),
    email_conj VARCHAR(255)
);