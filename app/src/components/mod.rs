pub mod form_custo;
pub mod form_pesagem;
pub mod seta_preco;
pub mod termometro;

use chrono::NaiveDate;
use domain::TipoCusto;

/// Formata um valor monetário para exibição. A UI não faz cálculo de negócio:
/// apenas converte o `f64` já calculado pelo domínio em texto.
pub(crate) fn formatar_moeda(valor: f64) -> String {
    let sinal = if valor < 0.0 { "-" } else { "" };
    format!("{sinal}R$ {:.2}", valor.abs()).replace('.', ",")
}

/// Exibe o resultado com sinal explícito, como pede o termômetro.
pub(crate) fn formatar_resultado(valor: f64) -> String {
    if valor >= 0.0 {
        format!("+{}", formatar_moeda(valor))
    } else {
        format!("-{}", formatar_moeda(valor.abs()))
    }
}

/// Converte a fração calculada pelo domínio (0.1 = 10%) em percentual legível.
pub(crate) fn formatar_percentual(valor: f64) -> String {
    format!("{:.1}%", valor * 100.0).replace('.', ",")
}

pub(crate) fn formatar_margem(valor: Option<f64>) -> String {
    valor.map_or_else(|| "—".to_string(), formatar_percentual)
}

pub(crate) fn formatar_gmd(valor: Option<f64>) -> String {
    valor.map_or_else(
        || "—".to_string(),
        |gmd| format!("{gmd:.2} kg/dia").replace('.', ","),
    )
}

pub(crate) fn formatar_peso(valor: f64) -> String {
    format!("{valor:.1} kg").replace('.', ",")
}

pub(crate) fn formatar_data(data: NaiveDate) -> String {
    data.format("%d/%m/%Y").to_string()
}

pub(crate) fn nome_tipo_custo(tipo: TipoCusto) -> &'static str {
    match tipo {
        TipoCusto::Sanidade => "Sanidade",
        TipoCusto::Suplemento => "Suplemento",
        TipoCusto::Pastagem => "Pastagem",
        TipoCusto::Outro => "Outro",
    }
}
