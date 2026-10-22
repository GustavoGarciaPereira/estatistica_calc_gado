pub mod custo;
pub mod lote;
pub mod pesagem;
pub mod preco;
pub mod resumo;

#[cfg(feature = "ssr")]
pub(crate) mod ssr {
    use axum::Extension;
    use leptos_axum::extract;
    use server_fn::ServerFnError;
    use sqlx::PgPool;

    /// O pool é injetado pelo `server` como `Extension<PgPool>` e recuperado
    /// pela extração do Axum dentro das server functions.
    pub(crate) async fn pool() -> Result<PgPool, ServerFnError> {
        let Extension(pool) = extract::<Extension<PgPool>>().await?;
        Ok(pool)
    }

    pub(crate) fn validar_status_lote(status: Option<String>) -> Result<(), ServerFnError> {
        match status.as_deref() {
            Some("ativo") => Ok(()),
            Some("vendido") => Err(ServerFnError::new(
                "lote já foi vendido e não aceita novos lançamentos",
            )),
            Some(outro) => Err(ServerFnError::new(format!(
                "status de lote desconhecido: {outro}"
            ))),
            None => Err(ServerFnError::new("lote não encontrado")),
        }
    }
}
