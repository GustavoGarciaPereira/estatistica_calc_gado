pub mod api;

use leptos::prelude::*;

/// Página inicial do Ciclo 2. O Ciclo 3 substitui pelo fluxo real.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <main>
            <h1>"Rentabilidade de Lotes"</h1>
            <p>"Fundação do servidor pronta. A tela de trabalho chega no Ciclo 3."</p>
        </main>
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
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <title>"Rentabilidade de Lotes"</title>
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
