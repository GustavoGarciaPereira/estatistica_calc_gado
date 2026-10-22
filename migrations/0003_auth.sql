CREATE TABLE usuarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    nome TEXT NOT NULL,
    senha_hash TEXT NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX usuarios_email_lower_idx ON usuarios (lower(email));

CREATE TABLE sessoes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    usuario_id UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expira_em TIMESTAMPTZ NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX sessoes_token_idx ON sessoes (token);

ALTER TABLE lotes
    ADD COLUMN usuario_id UUID NULL REFERENCES usuarios(id) ON DELETE CASCADE;

CREATE INDEX lotes_usuario_id_idx ON lotes (usuario_id);
