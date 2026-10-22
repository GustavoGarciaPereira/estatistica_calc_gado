-- Estrutura inicial do MVP: lote é a unidade de negócio, sempre com médias.

CREATE TABLE lotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nome TEXT NOT NULL,
    data_entrada DATE NOT NULL,
    qtd_animais INT NOT NULL CHECK (qtd_animais > 0),
    peso_inicial_medio NUMERIC NOT NULL CHECK (peso_inicial_medio > 0),
    custo_compra_total NUMERIC NOT NULL CHECK (custo_compra_total >= 0),
    status TEXT NOT NULL CHECK (status IN ('ativo', 'vendido'))
);

CREATE TABLE pesagens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lote_id UUID NOT NULL REFERENCES lotes(id) ON DELETE CASCADE,
    data DATE NOT NULL,
    peso_medio NUMERIC NOT NULL CHECK (peso_medio > 0)
);

CREATE INDEX pesagens_lote_data_idx ON pesagens (lote_id, data);

CREATE TABLE custos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lote_id UUID NOT NULL REFERENCES lotes(id) ON DELETE CASCADE,
    data DATE NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('sanidade', 'suplemento', 'pastagem', 'outro')),
    valor NUMERIC NOT NULL CHECK (valor >= 0)
);

CREATE INDEX custos_lote_data_idx ON custos (lote_id, data);

CREATE TABLE precos_mercado (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data DATE NOT NULL,
    regiao TEXT NOT NULL,
    preco_kg NUMERIC NOT NULL CHECK (preco_kg > 0),
    UNIQUE (data, regiao)
);
