#[cfg(feature = "ssr")]
use domain::resumo_lote as calcular_resumo;
use domain::{Custo, Lote, Pesagem, ResumoLote as ResumoLoteDomain};
use leptos::prelude::*;
use server_fn::ServerFnError;
use uuid::Uuid;

/// Região usada pela seta de preço enquanto o produto ainda não tem seleção
/// de região na interface.
#[cfg(feature = "ssr")]
const REGIAO_PADRAO: &str = "MT";

/// Resumo exibido na lista de lotes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LoteComResumo {
    pub lote: Lote,
    pub resumo: ResumoLoteDomain,
}

/// Dados completos da tela de detalhe do lote.
///
/// O resumo é calculado pelo domínio; pesagens e custos são apenas os
/// lançamentos usados para montar as listas da tela.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DetalheLote {
    pub lote: Lote,
    pub resumo: ResumoLoteDomain,
    pub pesagens: Vec<Pesagem>,
    pub custos: Vec<Custo>,
}

/// Variação do preço de mercado em relação ao preço anterior.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VariacaoPreco {
    pub preco_hoje: f64,
    pub preco_7d_atras: Option<f64>,
    pub variacao_percentual: Option<f64>,
}

#[server]
pub async fn resumo_lote(id: Uuid) -> Result<DetalheLote, ServerFnError> {
    use crate::api::custo::{row_para_custo, CustoRow};
    use crate::api::lote::{row_para_lote, LoteRow};
    use crate::api::pesagem::{row_para_pesagem, PesagemRow};
    use crate::api::preco::{row_para_preco, PrecoRow};
    use crate::api::ssr::pool;

    let pool = pool().await?;
    let data_ref = chrono::Local::now().date_naive();

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

    let regiao_row = sqlx::query_as!(RegiaoRow, "SELECT regiao FROM lotes WHERE id = $1", id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ServerFnError::new("lote não encontrado"))?;

    let regiao = regiao_row.regiao;

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
          AND data <= $2
        ORDER BY data DESC
        LIMIT 1
        "#,
        regiao.as_str(),
        data_ref
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| {
        ServerFnError::new(format!(
            "nenhum preço de mercado cadastrado para a região {regiao}"
        ))
    })?;

    let preco = row_para_preco(preco_row);
    let resumo = calcular_resumo(&lote, &pesagens, &custos, &preco, data_ref).map_err(|erro| {
        ServerFnError::new(format!("falha ao calcular o resumo do lote: {erro}"))
    })?;

    Ok(DetalheLote {
        lote,
        resumo,
        pesagens,
        custos,
    })
}

#[server]
pub async fn listar_lotes_com_resumo() -> Result<Vec<LoteComResumo>, ServerFnError> {
    use crate::api::lote::listar_lotes;

    // O resumo de cada lote é carregado individualmente (N+1). O limite de 50
    // mantém a tela previsível enquanto o MVP não tem paginação.
    const LIMITE_LOTES_COM_RESUMO: usize = 50;

    let lotes = listar_lotes().await?;
    let mut resultado = Vec::with_capacity(lotes.len().min(LIMITE_LOTES_COM_RESUMO));

    for lote in lotes.into_iter().take(LIMITE_LOTES_COM_RESUMO) {
        let id = Uuid::parse_str(&lote.id)
            .map_err(|_| ServerFnError::new("identificador de lote inválido no banco"))?;
        let detalhe = resumo_lote(id).await?;
        resultado.push(LoteComResumo {
            lote,
            resumo: detalhe.resumo,
        });
    }

    Ok(resultado)
}

#[server]
pub async fn variacao_preco_7d() -> Result<VariacaoPreco, ServerFnError> {
    use crate::api::ssr::pool;

    let pool = pool().await?;
    let data_ref = chrono::Local::now().date_naive();

    let row = sqlx::query_as!(
        VariacaoPrecoRow,
        r#"
        SELECT
            (
                SELECT preco_kg::float8
                FROM precos_mercado
                WHERE regiao = $1
                  AND data <= $2
                ORDER BY data DESC
                LIMIT 1
            ) AS "preco_hoje?",
            (
                SELECT preco_kg::float8
                FROM precos_mercado
                WHERE regiao = $1
                  AND data <= $2
                ORDER BY data DESC
                LIMIT 1 OFFSET 1
            ) AS "preco_7d_atras?"
        "#,
        REGIAO_PADRAO,
        data_ref
    )
    .fetch_one(&pool)
    .await?;

    let preco_hoje = row.preco_hoje.ok_or_else(|| {
        ServerFnError::new(format!(
            "nenhum preço de mercado cadastrado para a região {REGIAO_PADRAO}"
        ))
    })?;

    let variacao_percentual = match row.preco_7d_atras {
        Some(anterior) if anterior > 0.0 => Some((preco_hoje - anterior) / anterior),
        _ => None,
    };

    Ok(VariacaoPreco {
        preco_hoje,
        preco_7d_atras: row.preco_7d_atras,
        variacao_percentual,
    })
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct RegiaoRow {
    regiao: String,
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct VariacaoPrecoRow {
    preco_hoje: Option<f64>,
    preco_7d_atras: Option<f64>,
}
