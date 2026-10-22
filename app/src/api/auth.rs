#[cfg(feature = "ssr")]
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::ServerFnError;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Usuario {
    pub id: Uuid,
    pub email: String,
    pub nome: String,
}

#[server]
pub async fn cadastrar(nome: String, email: String, senha: String) -> Result<(), ServerFnError> {
    use crate::api::ssr::pool;

    let nome = nome.trim().to_string();
    let email = email.trim().to_lowercase();

    if nome.is_empty() {
        return Err(ServerFnError::new("Nome não pode ser vazio"));
    }

    if !email.contains('@') || !email.contains('.') {
        return Err(ServerFnError::new("Email inválido"));
    }

    if senha.chars().count() < 8 {
        return Err(ServerFnError::new(
            "Senha precisa ter pelo menos 8 caracteres",
        ));
    }

    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct UsuarioExistenteRow {
        id: Uuid,
    }

    let existente = sqlx::query_as!(
        UsuarioExistenteRow,
        "SELECT id FROM usuarios WHERE lower(email) = $1",
        email
    )
    .fetch_optional(&pool)
    .await?;

    if existente.map(|row| row.id).is_some() {
        return Err(ServerFnError::new("Email já cadastrado"));
    }

    let senha_hash = Argon2::default()
        .hash_password(senha.as_bytes())
        .map_err(|_| ServerFnError::new("Não foi possível processar a senha"))?
        .to_string();

    #[derive(sqlx::FromRow)]
    struct UsuarioIdRow {
        id: Uuid,
    }

    let usuario = sqlx::query_as!(
        UsuarioIdRow,
        r#"
        INSERT INTO usuarios (nome, email, senha_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        nome,
        email,
        senha_hash
    )
    .fetch_one(&pool)
    .await
    .map_err(|erro| {
        if erro
            .as_database_error()
            .and_then(|erro| erro.code())
            .as_deref()
            == Some("23505")
        {
            ServerFnError::new("Email já cadastrado")
        } else {
            ServerFnError::new("Não foi possível cadastrar o usuário")
        }
    })?;

    criar_sessao(usuario.id).await?;
    Ok(())
}

#[server]
pub async fn login(email: String, senha: String) -> Result<(), ServerFnError> {
    use crate::api::ssr::pool;

    let email = email.trim().to_lowercase();
    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct UsuarioLoginRow {
        id: Uuid,
        senha_hash: String,
    }

    let usuario = sqlx::query_as!(
        UsuarioLoginRow,
        "SELECT id, senha_hash FROM usuarios WHERE lower(email) = $1",
        email
    )
    .fetch_optional(&pool)
    .await?;

    let Some(usuario) = usuario else {
        return Err(ServerFnError::new("Email ou senha incorretos"));
    };

    let hash = PasswordHash::new(&usuario.senha_hash)
        .map_err(|_| ServerFnError::new("Email ou senha incorretos"))?;

    if Argon2::default()
        .verify_password(senha.as_bytes(), &hash)
        .is_err()
    {
        return Err(ServerFnError::new("Email ou senha incorretos"));
    }

    criar_sessao(usuario.id).await?;
    Ok(())
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::api::ssr::pool;
    use tower_sessions::Session;

    let session = leptos_axum::extract::<Session>().await?;
    let token = session
        .get::<String>("sessao_token")
        .await
        .map_err(|_| ServerFnError::new("Não foi possível encerrar a sessão"))?;

    if let Some(token) = token {
        let pool = pool().await?;

        #[derive(sqlx::FromRow)]
        struct SessaoTokenRow {
            token: String,
        }

        let _token_removido = sqlx::query_as!(
            SessaoTokenRow,
            "DELETE FROM sessoes WHERE token = $1 RETURNING token",
            token
        )
        .fetch_optional(&pool)
        .await?
        .map(|row| row.token);
    }

    session
        .flush()
        .await
        .map_err(|_| ServerFnError::new("Não foi possível encerrar a sessão"))?;

    Ok(())
}

#[server]
pub async fn usuario_atual() -> Result<Option<Usuario>, ServerFnError> {
    use crate::api::ssr::pool;

    let Some(usuario_id) = usuario_id_da_sessao().await? else {
        return Ok(None);
    };

    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct UsuarioRow {
        id: Uuid,
        email: String,
        nome: String,
    }

    let usuario = sqlx::query_as!(
        UsuarioRow,
        "SELECT id, email, nome FROM usuarios WHERE id = $1",
        usuario_id
    )
    .fetch_optional(&pool)
    .await?;

    Ok(usuario.map(|usuario| Usuario {
        id: usuario.id,
        email: usuario.email,
        nome: usuario.nome,
    }))
}

#[cfg(feature = "ssr")]
pub(crate) async fn usuario_id_da_sessao() -> Result<Option<Uuid>, ServerFnError> {
    use crate::api::ssr::pool;
    use tower_sessions::Session;

    let session = leptos_axum::extract::<Session>().await?;
    let token = session
        .get::<String>("sessao_token")
        .await
        .map_err(|_| ServerFnError::new("não autenticado"))?;

    let Some(token) = token else {
        return Ok(None);
    };

    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct SessaoRow {
        usuario_id: Uuid,
    }

    let sessao = sqlx::query_as!(
        SessaoRow,
        "SELECT usuario_id FROM sessoes WHERE token = $1 AND expira_em > now()",
        token
    )
    .fetch_optional(&pool)
    .await?;

    Ok(sessao.map(|sessao| sessao.usuario_id))
}

#[cfg(feature = "ssr")]
pub(crate) async fn exigir_usuario_id() -> Result<Uuid, ServerFnError> {
    usuario_id_da_sessao()
        .await?
        .ok_or_else(|| ServerFnError::new("não autenticado"))
}

#[cfg(feature = "ssr")]
async fn criar_sessao(usuario_id: Uuid) -> Result<(), ServerFnError> {
    use crate::api::ssr::pool;
    use chrono::{Duration, Utc};
    use rand::{distributions::Alphanumeric, Rng};
    use tower_sessions::Session;

    let session = leptos_axum::extract::<Session>().await?;
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let expira_em = Utc::now() + Duration::days(7);
    let pool = pool().await?;

    #[derive(sqlx::FromRow)]
    struct SessaoIdRow {
        id: Uuid,
    }

    let _sessao_id = sqlx::query_as!(
        SessaoIdRow,
        r#"
        INSERT INTO sessoes (usuario_id, token, expira_em)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        usuario_id,
        token,
        expira_em
    )
    .fetch_one(&pool)
    .await?
    .id;

    session
        .insert("usuario_id", usuario_id.to_string())
        .await
        .map_err(|_| ServerFnError::new("Não foi possível criar a sessão"))?;
    session
        .insert("sessao_token", token)
        .await
        .map_err(|_| ServerFnError::new("Não foi possível criar a sessão"))?;

    Ok(())
}
