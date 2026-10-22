use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use server_fn::ServerFnError;
use uuid::Uuid;

use crate::api::{resumo_lote, variacao_preco_7d, DetalheLote, VariacaoPreco};
use crate::components::form_custo::FormCusto;
use crate::components::form_pesagem::FormPesagem;
use crate::components::seta_preco::SetaPreco;
use crate::components::termometro::Termometro;
use crate::components::{formatar_data, formatar_moeda, formatar_peso, nome_tipo_custo};

type RecursoVariacao = Resource<Result<VariacaoPreco, ServerFnError>>;

#[component]
pub fn PaginaLoteDetalhe() -> impl IntoView {
    let params = use_params_map();
    let detalhe = Resource::new(
        move || params.read().get("id").unwrap_or_default(),
        |id| async move {
            match Uuid::parse_str(&id) {
                Ok(uuid) => resumo_lote(uuid).await,
                Err(_) => Err(ServerFnError::new("lote não encontrado")),
            }
        },
    );
    let variacao = Resource::new(|| (), |_| variacao_preco_7d());
    let ao_registrar = Callback::new(move |_| {
        detalhe.refetch();
        variacao.refetch();
    });

    view! {
        <main class="pagina">
            <header class="topo">
                <div>
                    <h1>"Detalhe do lote"</h1>
                    <p>"Acompanhe o termômetro e lance pesagens e custos."</p>
                </div>
                <A href="/lotes" attr:class="botao-secundario">"Voltar"</A>
            </header>

            <Suspense fallback=|| view! {
                <p class="carregando">"Carregando lote..."</p>
            }>
                {move || match detalhe.get() {
                    Some(Ok(detalhe)) => view! {
                        <ConteudoDetalhe
                            detalhe=detalhe
                            variacao=variacao
                            ao_registrar=ao_registrar
                        />
                    }.into_any(),
                    Some(Err(erro)) => view! {
                        <section class="estado-vazio">
                            <h2>"Lote não encontrado"</h2>
                            <p>{format!("{erro}")}</p>
                            <A href="/lotes" attr:class="botao">"Voltar para a lista"</A>
                        </section>
                    }.into_any(),
                    None => view! {
                        <p class="carregando">"Carregando lote..."</p>
                    }.into_any(),
                }}
            </Suspense>
        </main>
    }
}

#[component]
fn ConteudoDetalhe(
    detalhe: DetalheLote,
    variacao: RecursoVariacao,
    ao_registrar: Callback<()>,
) -> impl IntoView {
    let DetalheLote {
        lote,
        resumo,
        pesagens,
        custos,
    } = detalhe;

    let lote_id = Uuid::parse_str(&lote.id);
    let nome = lote.nome;
    let qtd_animais = lote.qtd_animais;
    let data_entrada = formatar_data(lote.data_entrada);
    let pesagens_vazias = pesagens.is_empty();
    let custos_vazios = custos.is_empty();

    view! {
        <section class="detalhe-lote">
            <div class="cabecalho-lote">
                <h2>{nome}</h2>
                <p>
                    {format!("{qtd_animais} animais • entrada em {data_entrada}")}
                </p>
            </div>

            <Termometro resumo=resumo/>

            <Suspense fallback=|| view! {
                <p class="carregando">"Carregando preço..."</p>
            }>
                {move || match variacao.get() {
                    Some(Ok(variacao)) => view! {
                        <SetaPreco variacao=variacao/>
                    }.into_any(),
                    Some(Err(erro)) => view! {
                        <p class="erro">{format!("Preço indisponível: {erro}")}</p>
                    }.into_any(),
                    None => view! {
                        <p class="carregando">"Carregando preço..."</p>
                    }.into_any(),
                }}
            </Suspense>

            {match lote_id {
                Ok(id) => view! {
                    <div class="grade-formularios">
                        <FormPesagem lote_id=id on_success=ao_registrar/>
                        <FormCusto lote_id=id on_success=ao_registrar/>
                    </div>
                }.into_any(),
                Err(_) => view! {
                    <p class="erro">"Identificador do lote inválido."</p>
                }.into_any(),
            }}

            <div class="listas">
                <section class="lista-lancamentos">
                    <h3>"Pesagens"</h3>
                    {pesagens_vazias.then(|| view! {
                        <p class="vazio">"Nenhuma pesagem registrada"</p>
                    })}
                    <ul>
                        {pesagens
                            .into_iter()
                            .map(|pesagem| view! {
                                <li>
                                    {formatar_data(pesagem.data)}
                                    " — "
                                    {formatar_peso(pesagem.peso_medio)}
                                </li>
                            })
                            .collect_view()}
                    </ul>
                </section>

                <section class="lista-lancamentos">
                    <h3>"Custos"</h3>
                    {custos_vazios.then(|| view! {
                        <p class="vazio">"Nenhum custo registrado"</p>
                    })}
                    <ul>
                        {custos
                            .into_iter()
                            .map(|custo| view! {
                                <li>
                                    {formatar_data(custo.data)}
                                    " — "
                                    {nome_tipo_custo(custo.tipo)}
                                    " — "
                                    {formatar_moeda(custo.valor)}
                                </li>
                            })
                            .collect_view()}
                    </ul>
                </section>
            </div>
        </section>
    }
}
