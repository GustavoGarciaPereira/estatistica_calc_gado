use leptos::prelude::*;

use crate::api::VariacaoPreco;

use super::{formatar_moeda, formatar_percentual};

#[component]
pub fn SetaPreco(variacao: VariacaoPreco) -> impl IntoView {
    let preco_hoje = format!("{}/kg", formatar_moeda(variacao.preco_hoje));

    let (simbolo, classe, texto) = match variacao.variacao_percentual {
        Some(valor) if valor > 0.01 => (
            "↑",
            "seta-alta",
            format!("({} em 7 dias)", formatar_percentual(valor)),
        ),
        Some(valor) if valor < -0.01 => (
            "↓",
            "seta-baixa",
            format!("({} em 7 dias)", formatar_percentual(valor)),
        ),
        Some(valor) => (
            "→",
            "seta-estavel",
            format!("({} em 7 dias)", formatar_percentual(valor)),
        ),
        None => (
            "",
            "seta-sem-historico",
            "sem histórico de 7 dias".to_string(),
        ),
    };

    view! {
        <div class="seta-preco">
            <span class="preco-atual">{preco_hoje}</span>
            <span class=format!("variacao {classe}")>
                {(!simbolo.is_empty()).then_some(simbolo)}
                " "
                {texto}
            </span>
        </div>
    }
}
