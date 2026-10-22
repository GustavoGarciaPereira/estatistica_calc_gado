use chrono::NaiveDate;
use domain::Pesagem;
use leptos::prelude::*;
use server_fn::ServerFnError;
use uuid::Uuid;

#[server]
pub async fn criar_pesagem(
    lote_id: Uuid,
    data: NaiveDate,
    peso_medio: f64,
) -> Result<Pesagem, ServerFnError> {
    use crate::api::ssr::{pool, validar_status_lote};

    if !peso_medio.is_finite() || peso_medio <= 0.0 {
        return Err(ServerFnError::new("peso médio deve ser maior que zero"));
    }

    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct StatusLoteRow {
        status: String,
    }

    let status = sqlx::query_as!(
        StatusLoteRow,
        "SELECT status FROM lotes WHERE id = $1",
        lote_id
    )
    .fetch_optional(&pool)
    .await?
    .map(|row| row.status);

    validar_status_lote(status)?;

    let row = sqlx::query_as!(
        PesagemRow,
        r#"
        INSERT INTO pesagens (lote_id, data, peso_medio)
        VALUES ($1, $2, $3::float8::numeric)
        RETURNING
            id,
            lote_id,
            data,
            peso_medio::float8 AS "peso_medio!"
        "#,
        lote_id,
        data,
        peso_medio
    )
    .fetch_one(&pool)
    .await?;

    Ok(row_para_pesagem(row))
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
pub(crate) struct PesagemRow {
    pub(crate) id: Uuid,
    pub(crate) lote_id: Uuid,
    pub(crate) data: NaiveDate,
    pub(crate) peso_medio: f64,
}

#[cfg(feature = "ssr")]
pub(crate) fn row_para_pesagem(row: PesagemRow) -> Pesagem {
    Pesagem {
        id: row.id.to_string(),
        lote_id: row.lote_id.to_string(),
        data: row.data,
        peso_medio: row.peso_medio,
    }
}
