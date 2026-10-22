use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

use crate::api::auth::{usuario_atual, Logout};

#[component]
pub fn Header() -> impl IntoView {
    let local = use_location();
    let usuario = Resource::new(move || local.pathname.get(), |_| usuario_atual());
    let acao = ServerAction::<Logout>::new();
    let navigate = use_navigate();
    let valor = acao.value();

    Effect::new(move |_| {
        if matches!(valor.get(), Some(Ok(()))) {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Suspense fallback=|| view! { <span></span> }>
            {move || {
                let path = local.pathname.get();
                if path == "/login" || path == "/cadastro" {
                    return None;
                }

                match usuario.get() {
                    Some(Ok(Some(usuario))) => Some(view! {
                        <header class="pagina topo barra-usuario">
                            <div>
                                <strong>{usuario.nome}</strong>
                            </div>
                            <ActionForm action=acao>
                                <button type="submit" class="botao-secundario">"Sair"</button>
                            </ActionForm>
                        </header>
                    }),
                    _ => None,
                }
            }}
        </Suspense>
    }
}
