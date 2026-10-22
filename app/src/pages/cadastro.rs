use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::api::auth::Cadastrar;

#[component]
pub fn PaginaCadastro() -> impl IntoView {
    let acao = ServerAction::<Cadastrar>::new();
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
                <h1>"Criar conta"</h1>
                <p>"Cadastre-se para começar a acompanhar seus lotes."</p>

                <ActionForm action=acao>
                    <label>
                        "Nome"
                        <input type="text" name="nome" autocomplete="name" required/>
                    </label>
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
                            autocomplete="new-password"
                            minlength="8"
                            required
                        />
                    </label>
                    <button type="submit" class="botao">"Criar conta"</button>
                </ActionForm>

                {move || match valor.get() {
                    Some(Err(erro)) => Some(view! {
                        <p class="erro">{format!("{erro}")}</p>
                    }),
                    _ => None,
                }}

                <p>
                    "Já tem conta? "
                    <A href="/login">"Entrar"</A>
                </p>
            </section>
        </main>
    }
}
