pub mod api;
pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::pages::{PaginaHome, PaginaLoteDetalhe, PaginaLotes, PaginaNovoLote};

/// O CSS é embutido no HTML do SSR para não depender de configuração extra de
/// assets estáticos neste ciclo.
#[cfg(feature = "ssr")]
const ESTILOS: &str = include_str!("styles.css");

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! {
                <main class="pagina">
                    <h1>"Página não encontrada"</h1>
                    <p>"O endereço acessado não existe."</p>
                    <A href="/lotes" attr:class="botao">"Ir para os lotes"</A>
                </main>
            }>
                <Route path=path!("/") view=PaginaHome/>
                <Route path=path!("/lotes") view=PaginaLotes/>
                <Route path=path!("/lotes/novo") view=PaginaNovoLote/>
                <Route path=path!("/lotes/:id") view=PaginaLoteDetalhe/>
            </Routes>
        </Router>
    }
}

/// Shell HTML usado pelo Axum no SSR.
#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="pt-BR">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" href="data:,"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <title>"Rentabilidade de Lotes"</title>
                <style>{ESTILOS}</style>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[cfg(feature = "ssr")]
pub use leptos::config::get_configuration;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use leptos::wasm_bindgen::prelude::*;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(App);
}
