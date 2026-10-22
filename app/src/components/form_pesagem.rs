use chrono::Local;
use leptos::form::ActionForm;
use leptos::prelude::*;
use uuid::Uuid;

use crate::api::pesagem::CriarPesagem;

#[component]
pub fn FormPesagem(lote_id: Uuid, on_success: Callback<()>) -> impl IntoView {
    let acao = ServerAction::<CriarPesagem>::new();
    let form_ref = NodeRef::<leptos::html::Form>::new();
    let hoje = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let valor = acao.value();

    Effect::new(move |_| {
        if let Some(Ok(_)) = valor.get() {
            #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
            {
                if let Some(form) = form_ref.get() {
                    form.reset();
                }
            }
            on_success.run(());
        }
    });

    view! {
        <section class="formulario">
            <h2>"Nova pesagem"</h2>
            <ActionForm action=acao node_ref=form_ref>
                <input type="hidden" name="lote_id" value=lote_id.to_string()/>
                <label>
                    "Data"
                    <input type="date" name="data" value=hoje required/>
                </label>
                <label>
                    "Peso médio (kg)"
                    <input
                        type="number"
                        name="peso_medio"
                        min="0.01"
                        step="0.01"
                        required
                    />
                </label>
                <button type="submit" class="botao">"Registrar pesagem"</button>
            </ActionForm>

            {move || match valor.get() {
                Some(Err(erro)) => Some(view! {
                    <p class="erro">{format!("{erro}")}</p>
                }),
                _ => None,
            }}
        </section>
    }
}
