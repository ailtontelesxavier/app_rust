-- Add migration script here
CREATE TABLE emprestimo_regiao (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    municipio_id INT NOT NULL REFERENCES municipio(id)
);
CREATE INDEX idx_emprestimo_regiao_name ON emprestimo_regiao(name);

CREATE TABLE emprestimo_regiao_cidades (
    id SERIAL PRIMARY KEY,
    regiao_id INT NOT NULL REFERENCES emprestimo_regiao(id),
    municipio_id INT NOT NULL REFERENCES municipio(id),
    CONSTRAINT emprestimo_regiao_cidades_unique UNIQUE (regiao_id, municipio_id)
);

CREATE TABLE emprestimo_user_regiao (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    regiao_id INT NOT NULL REFERENCES emprestimo_regiao(id),
    CONSTRAINT emprestimo_user_regiao_unique UNIQUE (user_id, regiao_id)
);

CREATE TABLE emprestimo_user_linha (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    linha_id INT NOT NULL REFERENCES linha(id),
    CONSTRAINT emprestimo_user_linha_unique UNIQUE (user_id, linha_id)
);
