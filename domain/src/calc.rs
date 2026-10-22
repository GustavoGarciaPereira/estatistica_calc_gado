use chrono::NaiveDate;

use crate::errors::DomainError;
use crate::models::{Custo, Lote, Pesagem, PrecoMercado, ResumoLote};

/// Soma o custo de compra do lote com os custos lançados para ele.
///
/// Custos de outros lotes são ignorados, mesmo que sejam passados na fatia.
pub fn calcular_custo_total(lote: &Lote, custos: &[Custo]) -> f64 {
    let custos_do_lote: f64 = custos
        .iter()
        .filter(|custo| custo.lote_id == lote.id)
        .map(|custo| custo.valor)
        .sum();

    lote.custo_compra_total + custos_do_lote
}

/// Retorna a pesagem mais recente do lote dentro da janela analisada.
fn ultima_pesagem(
    pesagens: &[Pesagem],
    data_entrada: NaiveDate,
    data_ref: NaiveDate,
) -> Option<&Pesagem> {
    pesagens
        .iter()
        .filter(|pesagem| pesagem.data >= data_entrada && pesagem.data <= data_ref)
        .max_by_key(|pesagem| pesagem.data)
}

/// Ganho médio diário em kg/dia.
///
/// Usa a pesagem mais recente entre a entrada e a data de referência. Sem
/// pesagem válida (ou se a única pesagem for no dia da entrada), retorna
/// `None` — o domínio nunca inventa GMD.
pub fn calcular_gmd(
    peso_inicial: f64,
    pesagens: &[Pesagem],
    data_entrada: NaiveDate,
    data_ref: NaiveDate,
) -> Option<f64> {
    let pesagem = ultima_pesagem(pesagens, data_entrada, data_ref)?;
    let dias = (pesagem.data - data_entrada).num_days();

    if dias <= 0 {
        return None;
    }

    Some((pesagem.peso_medio - peso_inicial) / dias as f64)
}

/// Peso médio atual do lote.
///
/// Usa a pesagem mais recente dentro da janela. Sem pesagem, assume o peso
/// inicial (decisão conservadora: não estima ganho sem dado real).
pub fn calcular_peso_atual(
    peso_inicial: f64,
    pesagens: &[Pesagem],
    data_entrada: NaiveDate,
    data_ref: NaiveDate,
) -> f64 {
    ultima_pesagem(pesagens, data_entrada, data_ref)
        .map_or(peso_inicial, |pesagem| pesagem.peso_medio)
}

/// Valor de mercado do lote inteiro: peso médio × quantidade × preço do kg.
pub fn calcular_valor_mercado(peso_atual: f64, qtd_animais: u32, preco_kg: f64) -> f64 {
    peso_atual * f64::from(qtd_animais) * preco_kg
}

/// Resultado se o lote fosse vendido hoje.
pub fn calcular_resultado(valor_mercado: f64, custo_total: f64) -> f64 {
    valor_mercado - custo_total
}

/// Margem sobre o custo total.
///
/// Custo total zero torna a margem indefinida; retorna `None` em vez de
/// `inf`/`NaN`.
pub fn calcular_margem(resultado: f64, custo_total: f64) -> Option<f64> {
    if custo_total == 0.0 {
        None
    } else {
        Some(resultado / custo_total)
    }
}

/// Termômetro completo do lote.
///
/// Erros de contrato:
/// - `DataInvalida` se `data_ref` for anterior à entrada;
/// - `ParametroInvalido` se a quantidade for zero ou o preço for inválido.
///
/// Pesagens de outros lotes e pesagens futuras são descartadas. Custos
/// posteriores à data de referência também são descartados.
pub fn resumo_lote(
    lote: &Lote,
    pesagens: &[Pesagem],
    custos: &[Custo],
    preco: &PrecoMercado,
    data_ref: NaiveDate,
) -> Result<ResumoLote, DomainError> {
    if data_ref < lote.data_entrada {
        return Err(DomainError::DataInvalida);
    }

    if lote.qtd_animais == 0 || !preco.preco_kg.is_finite() || preco.preco_kg <= 0.0 {
        return Err(DomainError::ParametroInvalido);
    }

    let pesagens_do_lote: Vec<Pesagem> = pesagens
        .iter()
        .filter(|pesagem| pesagem.lote_id == lote.id)
        .cloned()
        .collect();
    let custos_ate_ref: Vec<Custo> = custos
        .iter()
        .filter(|custo| custo.data <= data_ref)
        .cloned()
        .collect();

    let custo_total = calcular_custo_total(lote, &custos_ate_ref);
    let gmd = calcular_gmd(
        lote.peso_inicial_medio,
        &pesagens_do_lote,
        lote.data_entrada,
        data_ref,
    );
    let peso_atual = calcular_peso_atual(
        lote.peso_inicial_medio,
        &pesagens_do_lote,
        lote.data_entrada,
        data_ref,
    );
    let valor_mercado = calcular_valor_mercado(peso_atual, lote.qtd_animais, preco.preco_kg);
    let resultado = calcular_resultado(valor_mercado, custo_total);
    let margem = calcular_margem(resultado, custo_total);
    let dias_no_pasto = (data_ref - lote.data_entrada).num_days() as u32;

    Ok(ResumoLote {
        custo_total,
        gmd,
        peso_atual,
        valor_mercado,
        resultado,
        margem,
        dias_no_pasto,
    })
}
