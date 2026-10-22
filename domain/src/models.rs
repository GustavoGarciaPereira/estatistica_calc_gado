use chrono::NaiveDate;

/// Identificador opaco usado pelo domínio.
///
/// O crate não depende de `uuid`: a camada de persistência converte o UUID do
/// banco para string na borda e o domínio só compara identificadores.
pub type Id = String;

/// Lote é a unidade de negócio. Nunca há cálculo por indivíduo.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lote {
    pub id: Id,
    pub nome: String,
    pub data_entrada: NaiveDate,
    pub qtd_animais: u32,
    pub peso_inicial_medio: f64,
    pub custo_compra_total: f64,
    pub status: LoteStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoteStatus {
    Ativo,
    Vendido,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pesagem {
    pub id: Id,
    pub lote_id: Id,
    pub data: NaiveDate,
    pub peso_medio: f64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Custo {
    pub id: Id,
    pub lote_id: Id,
    pub data: NaiveDate,
    pub tipo: TipoCusto,
    pub valor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TipoCusto {
    Sanidade,
    Suplemento,
    Pastagem,
    Outro,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrecoMercado {
    pub data: NaiveDate,
    pub regiao: String,
    pub preco_kg: f64,
}

/// Resultado do termômetro de rentabilidade de um lote.
///
/// `gmd` e `margem` são `Option` porque os critérios de aceitação preveem
/// ausência de pesagem (GMD desconhecido) e custo total zero (margem indefinida).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResumoLote {
    pub custo_total: f64,
    pub gmd: Option<f64>,
    pub peso_atual: f64,
    pub valor_mercado: f64,
    pub resultado: f64,
    pub margem: Option<f64>,
    pub dias_no_pasto: u32,
}
