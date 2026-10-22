#[cfg(feature = "ssr")]
use domain::resumo_lote as calcular_resumo;
use domain::ResumoLote as ResumoLoteDomain;
use leptos::prelude::*;
use server_fn::ServerFnError;
use uuid::Uuid;

#[server]
pub async fn resumo_lote(id: Uuid) -> Result<ResumoLoteDomain, ServerFnError> {
    use crate::api::custo::{row_para_custo, CustoRow};
    use crate::api::lote::{row_para_lote, LoteRow};
    use crate::api::pesagem::{row_para_pesagem, PesagemRow};
    use crate::api::preco::{row_para_preco, PrecoRow};
    use crate::api::ssr::pool;

    const REGIAO_PADRAO: &str = "MT";

    let pool = pool().await?;

    let lote_row = sqlx::query_as!(
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
    .await?
    .ok_or_else(|| ServerFnError::new("lote não encontrado"))?;

    let lote = row_para_lote(lote_row)?;

    let pesagens = sqlx::query_as!(
        PesagemRow,
        r#"
        SELECT
            id,
            lote_id,
            data,
            peso_medio::float8 AS "peso_medio!"
        FROM pesagens
        WHERE lote_id = $1
        ORDER BY data
        "#,
        id
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(row_para_pesagem)
    .collect::<Vec<_>>();

    let custos = sqlx::query_as!(
        CustoRow,
        r#"
        SELECT
            id,
            lote_id,
            data,
            tipo,
            valor::float8 AS "valor!"
        FROM custos
        WHERE lote_id = $1
        ORDER BY data
        "#,
        id
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(row_para_custo)
    .collect::<Result<Vec<_>, _>>()?;

    let preco_row = sqlx::query_as!(
        PrecoRow,
        r#"
        SELECT
            data,
            regiao,
            preco_kg::float8 AS "preco_kg!"
        FROM precos_mercado
        WHERE regiao = $1
        ORDER BY data DESC
        LIMIT 1
        "#,
        REGIAO_PADRAO
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| {
        ServerFnError::new(format!(
            "nenhum preço de mercado cadastrado para a região {REGIAO_PADRAO}"
        ))
    })?;

    let preco = row_para_preco(preco_row);
    let data_ref = chrono::Utc::now().date_naive();

    calcular_resumo(&lote, &pesagens, &custos, &preco, data_ref)
        .map_err(|erro| ServerFnError::new(format!("falha ao calcular o resumo do lote: {erro}")))
}
