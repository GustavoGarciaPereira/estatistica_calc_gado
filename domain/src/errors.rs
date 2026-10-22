use std::error::Error;
use std::fmt;

/// Erros de contrato do domínio.
///
/// O Ciclo 1 expõe apenas os dois casos exigidos pelo Contrato de Aceitação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    /// A data de referência é anterior à data de entrada do lote.
    DataInvalida,
    /// Quantidade de animais ou preço de mercado inválido.
    ParametroInvalido,
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataInvalida => {
                formatter.write_str("data de referência anterior à data de entrada do lote")
            }
            Self::ParametroInvalido => formatter.write_str("parâmetro inválido para o cálculo"),
        }
    }
}

impl Error for DomainError {}
