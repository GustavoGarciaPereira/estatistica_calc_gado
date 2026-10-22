use domain::ResumoLote;
use leptos::prelude::*;

use super::{formatar_gmd, formatar_margem, formatar_moeda, formatar_peso, formatar_resultado};

/// A classificação visual da margem é uma regra de apresentação do contrato.
/// O valor em si vem sempre calculado pelo domínio.
pub(crate) fn classe_margem(margem: Option<f64>) -> &'static str {
    match margem {
        Some(valor) if valor >= 0.10 => "termometro-verde",
        Some(valor) if valor >= 0.0 => "termometro-amarelo",
        Some(_) => "termometro-vermelho",
        None => "termometro-cinza",
    }
}

#[component]
pub fn Termometro(resumo: ResumoLote) -> impl IntoView {
    let classe = classe_margem(resumo.margem);
    let alerta = resumo.margem.is_some_and(|margem| margem >= 0.10);

    let custo_total = formatar_moeda(resumo.custo_total);
    let gmd = formatar_gmd(resumo.gmd);
    let peso_atual = formatar_peso(resumo.peso_atual);
    let valor_mercado = formatar_moeda(resumo.valor_mercado);
    let resultado = formatar_resultado(resumo.resultado);
    let margem = formatar_margem(resumo.margem);
    let dias_no_pasto = format!("{} dias", resumo.dias_no_pasto);

    view! {
        <section class=format!("termometro {classe}") id="termometro">
            <div class="termometro-topo">
                <h2>"Termômetro do lote"</h2>
                {alerta.then(|| {
                    view! {
                        <p id="alerta-venda" class="alerta-venda" role="status">
                            "Ponto de venda atingido"
                        </p>
                    }
                })}
            </div>

            <dl class="termometro-campos">
                <div class="termometro-campo">
                    <dt>"Custo total"</dt>
                    <dd>{custo_total}</dd>
                </div>
                <div class="termometro-campo">
                    <dt>"GMD"</dt>
                    <dd>{gmd}</dd>
                </div>
                <div class="termometro-campo">
                    <dt>"Peso atual"</dt>
                    <dd>{peso_atual}</dd>
                </div>
                <div class="termometro-campo">
                    <dt>"Valor de mercado"</dt>
                    <dd>{valor_mercado}</dd>
                </div>
                <div class="termometro-campo">
                    <dt>"Resultado"</dt>
                    <dd>{resultado}</dd>
                </div>
                <div class="termometro-campo">
                    <dt>"Margem"</dt>
                    <dd>{margem}</dd>
                </div>
                <div class="termometro-campo">
                    <dt>"Dias no pasto"</dt>
                    <dd>{dias_no_pasto}</dd>
                </div>
            </dl>
        </section>
    }
}
