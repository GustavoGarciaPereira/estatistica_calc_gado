use std::error::Error;

use app::{get_configuration, shell};
use axum::{
    extract::{Extension, Request},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use leptos_axum::{file_and_error_handler, generate_route_list, LeptosRoutes};
use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;
use tower_sessions::{
    cookie::{Key, SameSite},
    MemoryStore, Session, SessionManagerLayer,
};

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

    let session_secret = std::env::var("SESSION_SECRET").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "SESSION_SECRET não definida; use pelo menos 64 caracteres",
        )
    })?;

    if session_secret.len() < 64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "SESSION_SECRET precisa ter pelo menos 64 caracteres",
        )
        .into());
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let session_key = Key::try_from(session_secret.as_bytes()).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "SESSION_SECRET inválida para assinatura de sessão",
        )
    })?;

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

    // MemoryStore é aceitável para o MVP: as sessões vivem enquanto o processo
    // estiver de pé. A tabela `sessoes` mantém o vínculo auditável e permite
    // invalidar o token no logout.
    let session_layer = SessionManagerLayer::new(MemoryStore::default())
        .with_name("session")
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_path("/")
        .with_secure(false)
        .with_signed(session_key);

    let app = Router::new()
        .leptos_routes(&leptos_options, rotas, {
            let options = leptos_options.clone();
            move || shell(options.clone())
        })
        .nest_service("/pkg", ServeDir::new(diretorio_pkg))
        .fallback(file_and_error_handler(shell))
        .layer(middleware::from_fn(proteger_rotas))
        .layer(session_layer)
        .layer(Extension(pool))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(endereco).await?;
    tracing::info!("listening on http://{endereco}");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn proteger_rotas(session: Session, request: Request, next: Next) -> Response {
    if request.uri().path() == "/" {
        return (StatusCode::FOUND, [(header::LOCATION, "/lotes")]).into_response();
    }

    let protegida = request.uri().path() == "/lotes" || request.uri().path().starts_with("/lotes/");

    if !protegida {
        return next.run(request).await;
    }

    match session.get::<String>("usuario_id").await {
        Ok(Some(_)) => next.run(request).await,
        _ => (StatusCode::FOUND, [(header::LOCATION, "/login")]).into_response(),
    }
}
