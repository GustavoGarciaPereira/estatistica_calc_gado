use leptos::prelude::*;

use super::PaginaLotes;

/// A home do MVP é a própria lista de lotes. O produtor cai direto na
/// pergunta que importa: qual lote está no lucro ou no prejuízo.
#[component]
pub fn PaginaHome() -> impl IntoView {
    view! {
        <PaginaLotes/>
    }
}
