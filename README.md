# Rentabilidade de Lotes

Aplicação full-stack em Rust para o produtor saber, em poucos segundos, se o lote
está no lucro ou no prejuízo. O Ciclo 2 entrega a fundação: workspace Leptos +
Axum, PostgreSQL, migrações SQLx e as server functions de persistência.

## Pré-requisitos

- Rust stable com o target `wasm32-unknown-unknown`:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos) 0.3:
  ```bash
  cargo install cargo-leptos --locked
  ```
- [`sqlx-cli`](https://github.com/launchbadge/sqlx) 0.8:
  ```bash
  cargo install sqlx-cli --version 0.8.6 --locked \
    --no-default-features --features postgres,rustls
  ```
- Docker com Docker Compose.

## Configuração

Copie o arquivo de exemplo e ajuste as credenciais se quiser:

```bash
cp .env.example .env
```

O `.env` não é versionado. `.env.example` é o contrato das variáveis esperadas.

## Banco de dados

```bash
docker compose up -d
sqlx database create
sqlx migrate run
```

O serviço `postgres` usa a imagem `postgres:16-alpine`, persiste os dados no
volume `pgdata` e expõe a porta `5432`.

Para parar:

```bash
docker compose down
```

Para apagar também os dados locais:

```bash
docker compose down -v
```

## Rodar a aplicação

```bash
cargo leptos watch
```

A aplicação sobe em `http://127.0.0.1:3000`. Neste ciclo, a raiz mostra apenas
um placeholder; as telas reais entram no Ciclo 3.

O servidor Axum injeta o `sqlx::PgPool` como `Extension` e as server functions
ficam em `app/src/api/`, todas com `#[server]`. Toda query usa `sqlx::query_as!`
e os cálculos são delegados ao crate `domain/`.

## SQLx offline

Depois de subir o banco e rodar as migrações, gere os metadados de compilação:

```bash
cargo sqlx prepare --workspace
```

O diretório `.sqlx/` é versionado de propósito. Com ele, os builds funcionam sem
banco e sem `DATABASE_URL`:

```bash
SQLX_OFFLINE=true cargo build -p app --features ssr
SQLX_OFFLINE=true cargo build -p server
```

Sempre que uma query SQL mudar, rode `cargo sqlx prepare --workspace` novamente
e commite o `.sqlx/` atualizado.

## Verificação

```bash
cargo test -p domain
cargo clippy -p domain -- -D warnings
cargo fmt --all -- --check

SQLX_OFFLINE=true cargo build -p app --features ssr
SQLX_OFFLINE=true cargo build -p server
cargo clippy -p app --features ssr -- -D warnings
cargo clippy -p server -- -D warnings
```

## Estrutura

```text
.
├── app/               # Leptos: página placeholder + server functions
├── domain/            # Domínio puro, congelado no Ciclo 2
├── server/            # Entry point Axum (SSR) com PgPool via Extension
├── migrations/        # Migrações SQLx
├── .sqlx/             # Metadados de queries para build offline
├── docker-compose.yml
└── Cargo.toml         # Workspace
```

## Notas de implementação

- `NUMERIC` é convertido para `f64` nas queries com casts explícitos para
  `float8`, preservando o contrato do domínio sem adicionar tipos decimais.
- O preço de mercado usado no resumo é o mais recente da região constante
  `"MT"`; a seleção de região entra em um ciclo posterior.
- Sem preço cadastrado para a região, `resumo_lote` retorna erro claro.
