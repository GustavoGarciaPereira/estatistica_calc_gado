use chrono::NaiveDate;
#[cfg(feature = "ssr")]
use domain::PrecoMercado;
use leptos::prelude::*;
use server_fn::ServerFnError;
#[cfg(feature = "ssr")]
use uuid::Uuid;

#[server]
pub async fn upsert_preco(
    data: NaiveDate,
    regiao: String,
    preco_kg: f64,
) -> Result<(), ServerFnError> {
    use crate::api::auth::exigir_usuario_id;
    use crate::api::ssr::pool;

    let _usuario_id = exigir_usuario_id().await?;

    if regiao.trim().is_empty() {
        return Err(ServerFnError::new("região do preço não pode ser vazia"));
    }

    if !preco_kg.is_finite() || preco_kg <= 0.0 {
        return Err(ServerFnError::new("preço por kg deve ser maior que zero"));
    }

    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct IdRow {
        id: Uuid,
    }

    let row = sqlx::query_as!(
        IdRow,
        r#"
        INSERT INTO precos_mercado (data, regiao, preco_kg)
        VALUES ($1, $2, $3::float8::numeric)
        ON CONFLICT (data, regiao)
        DO UPDATE SET preco_kg = EXCLUDED.preco_kg
        RETURNING id
        "#,
        data,
        regiao,
        preco_kg
    )
    .fetch_one(&pool)
    .await?;

    let _ = row.id;
    Ok(())
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
pub(crate) struct PrecoRow {
    pub(crate) data: NaiveDate,
    pub(crate) regiao: String,
    pub(crate) preco_kg: f64,
}

#[cfg(feature = "ssr")]
pub(crate) fn row_para_preco(row: PrecoRow) -> PrecoMercado {
    PrecoMercado {
        data: row.data,
        regiao: row.regiao,
        preco_kg: row.preco_kg,
    }
}
