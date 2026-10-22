use std::error::Error;

use app::{get_configuration, shell};
use axum::{extract::Extension, Router};
use leptos_axum::{file_and_error_handler, generate_route_list, LeptosRoutes};
use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    if let Err(erro) = dotenvy::dotenv() {
        tracing::debug!("arquivo .env não carregado: {erro}");
    }

    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "DATABASE_URL não definida; copie .env.example para .env",
        )
    })?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let configuracao = get_configuration(Some("Cargo.toml"))?;
    let leptos_options = configuracao.leptos_options;
    let endereco = leptos_options.site_addr;

    let rotas = generate_route_list({
        let options = leptos_options.clone();
        move || shell(options.clone())
    });

    let diretorio_pkg = format!(
        "{}/{}",
        leptos_options.site_root, leptos_options.site_pkg_dir
    );

    let app = Router::new()
        .leptos_routes(&leptos_options, rotas, {
            let options = leptos_options.clone();
            move || shell(options.clone())
        })
        .nest_service("/pkg", ServeDir::new(diretorio_pkg))
        .fallback(file_and_error_handler(shell))
        .layer(Extension(pool))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(endereco).await?;
    tracing::info!("listening on http://{endereco}");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
