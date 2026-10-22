//! Domínio puro do produto de rentabilidade de lotes.
//!
//! Este crate não conhece interface gráfica, HTTP, banco de dados ou I/O.
//! Toda regra de negócio que decide se um lote está no lucro ou no prejuízo
//! vive aqui e é testada com `cargo test -p domain`.

pub mod calc;
pub mod errors;
pub mod models;

#[cfg(test)]
mod calc_test;

pub use calc::{
    calcular_custo_total, calcular_gmd, calcular_margem, calcular_peso_atual, calcular_resultado,
    calcular_valor_mercado, resumo_lote,
};
pub use errors::DomainError;
pub use models::{Custo, Id, Lote, LoteStatus, Pesagem, PrecoMercado, ResumoLote, TipoCusto};
