-- Add migration script here
-- Criação da tabela role_permissions (tabela associativa)
CREATE TABLE IF NOT EXISTS role_permissions (
    id SERIAL PRIMARY KEY,
    role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INT NOT NULL REFERENCES permission(id) ON DELETE CASCADE,
    CONSTRAINT unique_role_permission UNIQUE (role_id, permission_id)
);

-- Criação da tabela user_roles (usuário ↔ roles)
CREATE TABLE IF NOT EXISTS user_roles (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    CONSTRAINT unique_user_role UNIQUE (user_id, role_id)
);