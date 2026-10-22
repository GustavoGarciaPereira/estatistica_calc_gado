use chrono::NaiveDate;
use domain::{Custo, TipoCusto};
use leptos::prelude::*;
use server_fn::ServerFnError;
use uuid::Uuid;

#[server]
pub async fn criar_custo(
    lote_id: Uuid,
    data: NaiveDate,
    tipo: TipoCusto,
    valor: f64,
) -> Result<Custo, ServerFnError> {
    use crate::api::auth::exigir_usuario_id;
    use crate::api::ssr::{pool, validar_status_lote};

    let usuario_id = exigir_usuario_id().await?;

    if !valor.is_finite() || valor < 0.0 {
        return Err(ServerFnError::new("valor do custo não pode ser negativo"));
    }

    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct StatusLoteRow {
        status: String,
    }

    let status = sqlx::query_as!(
        StatusLoteRow,
        "SELECT status FROM lotes WHERE id = $1 AND usuario_id = $2",
        lote_id,
        usuario_id
    )
    .fetch_optional(&pool)
    .await?
    .map(|row| row.status);

    validar_status_lote(status)?;

    let tipo_banco = tipo_para_banco(tipo);
    let row = sqlx::query_as!(
        CustoRow,
        r#"
        INSERT INTO custos (lote_id, data, tipo, valor)
        VALUES ($1, $2, $3, $4::float8::numeric)
        RETURNING
            id,
            lote_id,
            data,
            tipo,
            valor::float8 AS "valor!"
        "#,
        lote_id,
        data,
        tipo_banco,
        valor
    )
    .fetch_one(&pool)
    .await?;

    row_para_custo(row)
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
pub(crate) struct CustoRow {
    pub(crate) id: Uuid,
    pub(crate) lote_id: Uuid,
    pub(crate) data: NaiveDate,
    pub(crate) tipo: String,
    pub(crate) valor: f64,
}

#[cfg(feature = "ssr")]
pub(crate) fn row_para_custo(row: CustoRow) -> Result<Custo, ServerFnError> {
    let tipo = tipo_do_banco(&row.tipo)?;

    Ok(Custo {
        id: row.id.to_string(),
        lote_id: row.lote_id.to_string(),
        data: row.data,
        tipo,
        valor: row.valor,
    })
}

#[cfg(feature = "ssr")]
fn tipo_para_banco(tipo: TipoCusto) -> &'static str {
    match tipo {
        TipoCusto::Sanidade => "sanidade",
        TipoCusto::Suplemento => "suplemento",
        TipoCusto::Pastagem => "pastagem",
        TipoCusto::Outro => "outro",
    }
}

#[cfg(feature = "ssr")]
fn tipo_do_banco(tipo: &str) -> Result<TipoCusto, ServerFnError> {
    match tipo {
        "sanidade" => Ok(TipoCusto::Sanidade),
        "suplemento" => Ok(TipoCusto::Suplemento),
        "pastagem" => Ok(TipoCusto::Pastagem),
        "outro" => Ok(TipoCusto::Outro),
        outro => Err(ServerFnError::new(format!(
            "tipo de custo desconhecido no banco: {outro}"
        ))),
    }
}
