use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::api::auth::Login;

#[component]
pub fn PaginaLogin() -> impl IntoView {
    let acao = ServerAction::<Login>::new();
    let navigate = use_navigate();
    let valor = acao.value();

    Effect::new(move |_| {
        if matches!(valor.get(), Some(Ok(()))) {
            navigate("/lotes", Default::default());
        }
    });

    view! {
        <main class="pagina">
            <section class="formulario">
                <h1>"Entrar"</h1>
                <p>"Acesse sua conta para acompanhar a rentabilidade dos lotes."</p>

                <ActionForm action=acao>
                    <label>
                        "Email"
                        <input
                            type="email"
                            name="email"
                            autocomplete="email"
                            required
                        />
                    </label>
                    <label>
                        "Senha"
                        <input
                            type="password"
                            name="senha"
                            autocomplete="current-password"
                            minlength="8"
                            required
                        />
                    </label>
                    <button type="submit" class="botao">"Entrar"</button>
                </ActionForm>

                {move || match valor.get() {
                    Some(Err(erro)) => Some(view! {
                        <p class="erro">{format!("{erro}")}</p>
                    }),
                    _ => None,
                }}

                <p>
                    "Ainda não tem conta? "
                    <A href="/cadastro">"Criar conta"</A>
                </p>
            </section>
        </main>
    }
}
