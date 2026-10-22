use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::{listar_lotes_com_resumo, LoteComResumo};
use crate::components::termometro::classe_margem;
use crate::components::{formatar_margem, formatar_moeda, formatar_resultado};

#[component]
pub fn PaginaLotes() -> impl IntoView {
    let lotes = Resource::new(|| (), |_| listar_lotes_com_resumo());

    view! {
        <main class="pagina pagina-lista">
            <header class="topo">
                <div>
                    <h1>"Lotes"</h1>
                    <p>"Rentabilidade do rebanho por lote"</p>
                </div>
                <A href="/lotes/novo" attr:class="botao botao-topo">"Novo Lote"</A>
            </header>

            <Suspense fallback=|| view! {
                <p class="carregando">"Carregando lotes..."</p>
            }>
                {move || match lotes.get() {
                    Some(Ok(lista)) => view! {
                        <ListaLotes lotes=lista/>
                    }.into_any(),
                    Some(Err(erro)) => view! {
                        <p class="erro">
                            {format!("Não foi possível carregar os lotes: {erro}")}
                        </p>
                    }.into_any(),
                    None => view! {
                        <p class="carregando">"Carregando lotes..."</p>
                    }.into_any(),
                }}
            </Suspense>
        </main>
    }
}

#[component]
fn ListaLotes(lotes: Vec<LoteComResumo>) -> impl IntoView {
    if lotes.is_empty() {
        view! {
            <section class="estado-vazio">
                <p>"Nenhum lote cadastrado"</p>
                <A href="/lotes/novo" attr:class="botao botao-estado-vazio">"Novo Lote"</A>
            </section>
        }
        .into_any()
    } else {
        view! {
            <div class="tabela-container">
                <table class="tabela-lotes">
                    <thead>
                        <tr>
                            <th scope="col">"Lote"</th>
                            <th scope="col">"Animais"</th>
                            <th scope="col">"Custo total"</th>
                            <th scope="col">"Resultado"</th>
                            <th scope="col">"Margem"</th>
                            <th scope="col">"Ações"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {lotes
                            .into_iter()
                            .map(|item| {
                                let LoteComResumo { lote, resumo } = item;
                                let classe_resultado = if resumo.resultado >= 0.0 {
                                    "resultado-positivo"
                                } else {
                                    "resultado-negativo"
                                };
                                view! {
                                    <tr>
                                        <td>{lote.nome}</td>
                                        <td>{lote.qtd_animais}</td>
                                        <td>{formatar_moeda(resumo.custo_total)}</td>
                                        <td class=classe_resultado>
                                            {formatar_resultado(resumo.resultado)}
                                        </td>
                                        <td>
                                            <span class=classe_margem(resumo.margem)>
                                                {formatar_margem(resumo.margem)}
                                            </span>
                                        </td>
                                        <td>
                                            <A
                                                href=format!("/lotes/{}", lote.id)
                                                attr:class="link-detalhe"
                                            >
                                                "Ver detalhe"
                                            </A>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        }
        .into_any()
    }
}
