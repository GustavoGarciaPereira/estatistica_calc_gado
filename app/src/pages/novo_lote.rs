use chrono::Local;
use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::api::lote::CriarLote;

#[component]
pub fn PaginaNovoLote() -> impl IntoView {
    let acao = ServerAction::<CriarLote>::new();
    let navigate = use_navigate();
    let valor = acao.value();
    let hoje = Local::now().date_naive().format("%Y-%m-%d").to_string();

    Effect::new(move |_| {
        if matches!(valor.get(), Some(Ok(_))) {
            navigate("/lotes", Default::default());
        }
    });

    view! {
        <main class="pagina">
            <header class="topo">
                <div>
                    <h1>"Novo lote"</h1>
                    <p>"Cadastre o lote para acompanhar a rentabilidade."</p>
                </div>
                <A href="/lotes" attr:class="botao-secundario">"Voltar"</A>
            </header>

            <section class="formulario">
                <ActionForm action=acao>
                    <label>
                        "Nome"
                        <input type="text" name="nome" required/>
                    </label>
                    <label>
                        "Data de entrada"
                        <input type="date" name="data_entrada" value=hoje required/>
                    </label>
                    <label>
                        "Quantidade de animais"
                        <input
                            type="number"
                            name="qtd_animais"
                            min="1"
                            step="1"
                            required
                        />
                    </label>
                    <label>
                        "Peso inicial médio (kg)"
                        <input
                            type="number"
                            name="peso_inicial_medio"
                            min="0.01"
                            step="0.01"
                            required
                        />
                    </label>
                    <label>
                        "Custo de compra total (R$)"
                        <input
                            type="number"
                            name="custo_compra_total"
                            min="0"
                            step="0.01"
                            required
                        />
                    </label>
                    <button type="submit" class="botao">"Cadastrar lote"</button>
                </ActionForm>

                {move || match valor.get() {
                    Some(Err(erro)) => Some(view! {
                        <p class="erro">{format!("{erro}")}</p>
                    }),
                    _ => None,
                }}
            </section>
        </main>
    }
}
