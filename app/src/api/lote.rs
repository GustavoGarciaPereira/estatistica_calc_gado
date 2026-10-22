use chrono::NaiveDate;
use domain::Lote;
use leptos::prelude::*;
use server_fn::ServerFnError;
use uuid::Uuid;

#[server]
pub async fn criar_lote(
    nome: String,
    data_entrada: NaiveDate,
    qtd_animais: u32,
    peso_inicial_medio: f64,
    custo_compra_total: f64,
) -> Result<Lote, ServerFnError> {
    use crate::api::ssr::pool;

    if nome.trim().is_empty() {
        return Err(ServerFnError::new("nome do lote não pode ser vazio"));
    }

    let qtd_animais = i32::try_from(qtd_animais)
        .map_err(|_| ServerFnError::new("quantidade de animais inválida"))?;

    if qtd_animais <= 0 {
        return Err(ServerFnError::new(
            "quantidade de animais deve ser maior que zero",
        ));
    }

    if !peso_inicial_medio.is_finite() || peso_inicial_medio <= 0.0 {
        return Err(ServerFnError::new(
            "peso inicial médio deve ser maior que zero",
        ));
    }

    if !custo_compra_total.is_finite() || custo_compra_total < 0.0 {
        return Err(ServerFnError::new(
            "custo de compra total não pode ser negativo",
        ));
    }

    let pool = pool().await?;
    let row = sqlx::query_as!(
        LoteRow,
        r#"
        INSERT INTO lotes (
            nome,
            data_entrada,
            qtd_animais,
            peso_inicial_medio,
            custo_compra_total,
            status
        )
        VALUES ($1, $2, $3, $4::float8::numeric, $5::float8::numeric, 'ativo')
        RETURNING
            id,
            nome,
            data_entrada,
            qtd_animais,
            peso_inicial_medio::float8 AS "peso_inicial_medio!",
            custo_compra_total::float8 AS "custo_compra_total!",
            status
        "#,
        nome,
        data_entrada,
        qtd_animais,
        peso_inicial_medio,
        custo_compra_total
    )
    .fetch_one(&pool)
    .await?;

    row_para_lote(row)
}

#[server]
pub async fn listar_lotes() -> Result<Vec<Lote>, ServerFnError> {
    use crate::api::ssr::pool;

    let pool = pool().await?;
    let rows = sqlx::query_as!(
        LoteRow,
        r#"
        SELECT
            id,
            nome,
            data_entrada,
            qtd_animais,
            peso_inicial_medio::float8 AS "peso_inicial_medio!",
            custo_compra_total::float8 AS "custo_compra_total!",
            status
        FROM lotes
        ORDER BY data_entrada DESC, nome ASC
        "#
    )
    .fetch_all(&pool)
    .await?;

    rows.into_iter().map(row_para_lote).collect()
}

#[server]
pub async fn buscar_lote(id: Uuid) -> Result<Option<Lote>, ServerFnError> {
    use crate::api::ssr::pool;

    let pool = pool().await?;
    let row = sqlx::query_as!(
        LoteRow,
        r#"
        SELECT
            id,
            nome,
            data_entrada,
            qtd_animais,
            peso_inicial_medio::float8 AS "peso_inicial_medio!",
            custo_compra_total::float8 AS "custo_compra_total!",
            status
        FROM lotes
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?;

    row.map(row_para_lote).transpose()
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
pub(crate) struct LoteRow {
    pub(crate) id: Uuid,
    pub(crate) nome: String,
    pub(crate) data_entrada: NaiveDate,
    pub(crate) qtd_animais: i32,
    pub(crate) peso_inicial_medio: f64,
    pub(crate) custo_compra_total: f64,
    pub(crate) status: String,
}

#[cfg(feature = "ssr")]
pub(crate) fn row_para_lote(row: LoteRow) -> Result<Lote, ServerFnError> {
    let qtd_animais = u32::try_from(row.qtd_animais)
        .map_err(|_| ServerFnError::new("quantidade de animais inválida no banco"))?;

    let status = match row.status.as_str() {
        "ativo" => domain::LoteStatus::Ativo,
        "vendido" => domain::LoteStatus::Vendido,
        outro => {
            return Err(ServerFnError::new(format!(
                "status de lote desconhecido no banco: {outro}"
            )));
        }
    };

    Ok(Lote {
        id: row.id.to_string(),
        nome: row.nome,
        data_entrada: row.data_entrada,
        qtd_animais,
        peso_inicial_medio: row.peso_inicial_medio,
        custo_compra_total: row.custo_compra_total,
        status,
    })
}
