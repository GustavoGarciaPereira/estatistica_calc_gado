# AGENTS.md — Rentabilidade de Lotes

> Fonte de verdade para qualquer agente (OpenCode, DSH ou humano) que trabalhe neste repositório.
> Leia este arquivo antes de codar. Em caso de conflito, o Contrato de Aceitação do ciclo vence este documento.

## 1. Tese do produto

O produtor abre o celular, no curral, e em 10 segundos sabe se o lote dele está no lucro ou no prejuízo — e se é hora de vender.

Toda decisão de código, feature ou tela passa por esse filtro. Se não ajuda a responder essa pergunta, fica para a v2 — depois do primeiro pagante.

## 2. Papéis e gate de qualidade

| Papel | Ferramenta | Responsabilidade |
| :--- | :--- | :--- |
| Especificador | Humano | Escreve o Contrato de Aceitação numerado e os prompts de implementação/auditoria |
| Implementador | OpenCode | Implementa para satisfazer o contrato, sem expandir escopo |
| Auditor | DeepSeek-Harness (DSH) | Verifica o código contra o mesmo contrato, sem implementar nem refatorar |
| Aprovador | Humano | Lê o veredito, roda os comandos e decide se o ciclo está aprovado |

Regras do loop:

1. O Contrato de Aceitação é numerado (`CA-1`, `CA-2`, ...), testável e **byte-idêntico** no prompt do implementador e no prompt do auditor.
2. O auditor não vê nem altera o código antes de receber o contrato.
3. O auditor não implementa. Sua saída é um veredito com evidência (`arquivo:linha` ou saída de comando) por CA.
4. Se reprovado, apenas os CAs apontados voltam ao implementador.
5. Se aprovado, o humano roda a verificação final e valida o comportamento.

## 3. Escopo travado do MVP

### Entra

- Cadastro de lote (nome, quantidade, peso inicial, custo de compra, data de entrada).
- Registro de pesagens (peso médio do lote, data).
- Registro de custos (tipo, valor, data).
- Termômetro de rentabilidade: custo total, GMD, peso atual, valor de mercado, resultado e margem.
- Preço de mercado manual e seta de variação de 7 dias.
- Alerta simples de margem-alvo (`margem >= limiar`).
- Registro por áudio → transcrição → LLM → gravação de pesagem/custo.

### NÃO FAÇA

1. **Sem tabela ou campo por indivíduo** (`Animal`, `Reproducao`, `Genealogia`, brinco, RFID). A unidade é sempre o **lote**, com médias.
2. **Sem dashboard, relatório em PDF ou gráfico zootécnico.** O máximo é o termômetro + sparkline de preço.
3. **Sem integração com balança, hardware, RFID ou API externa de preço.** Entrada manual ou por áudio.
4. **Sem wizard de onboarding multi-etapas.** Um lote em uma tela; resultado em outra.
5. **Sem feature fora do ciclo atual**, mesmo que pareça fácil ou "óbvia".
6. **Sem SQL fora de `#[server]`.** O frontend nunca vê SQL.
7. **Sem cálculo de negócio na UI.** A UI apenas exibe o que o crate `domain/` calculou.
8. **Sem `todo!()`, `unimplemented!()` ou `dbg!()`** no código entregue.
9. **Sem stub vazio ou função placeholder.** Se não tem comportamento real, não entra. Uma `fn vazia() {}` ou página sem conteúdo reprova mesmo que o nome não seja `todo!()`.
10. **Sem alterar `domain/` em ciclos posteriores** sem um contrato que autorize explicitamente.
11. **Sem inflar o ciclo.** Se não está no Contrato de Aceitação, não entra.

## 4. Modelo de domínio (Ciclo 1)

O crate `domain/` é Rust puro: sem UI, HTTP, banco, async ou I/O.
Dependências permitidas: `chrono` e `serde` opcional.

```text
Lote        { id, nome, data_entrada, qtd_animais, peso_inicial_medio,
              custo_compra_total, status: Ativo | Vendido }
Pesagem     { id, lote_id, data, peso_medio }
Custo       { id, lote_id, data, tipo: Sanidade | Suplemento | Pastagem | Outro, valor }
PrecoMercado{ data, regiao, preco_kg }
ResumoLote  { custo_total, gmd, peso_atual, valor_mercado,
              resultado, margem, dias_no_pasto }
```

Decisões do Ciclo 1:

- Campos monetários e de peso usam `f64` **por exigência do contrato atual**. Antes de cobrança/contabilidade real, reavaliar `Decimal` ou centavos.
- `Id` no domínio é `String` opaca para não acoplar o crate a `uuid`. A persistência converte UUID ↔ string na borda.
- `ResumoLote.gmd` e `ResumoLote.margem` são `Option<f64>`: GMD pode ser desconhecido e margem pode ser indefinida.
- Sem pesagem: `peso_atual = peso_inicial_medio` e `gmd = None`. O domínio **não inventa** GMD.
- A pesagem mais recente é a de maior `data` dentro de `[data_entrada, data_ref]`.
- Pesagens de outros lotes e pesagens futuras são ignoradas.
- Custos posteriores à `data_ref` são ignorados no resumo.
- `custo_total = custo_compra_total + SUM(custos do lote)`.
- `valor_mercado = peso_atual * qtd_animais * preco_kg`.
- `resultado = valor_mercado - custo_total`.
- `margem = resultado / custo_total`; custo total zero → `None`, nunca `inf`/`NaN`.
- `dias_no_pasto = (data_ref - data_entrada).num_days() as u32`.
- `data_ref < data_entrada` → `DomainError::DataInvalida`.
- `qtd_animais == 0` ou `preco_kg <= 0` → `DomainError::ParametroInvalido`.

## 5. Arquitetura alvo

```text
projeto/
├── AGENTS.md
├── Cargo.toml            # workspace
├── domain/               # Rust puro: modelos, cálculos e testes
├── app/                  # Leptos: UI + app/src/api/ com #[server] + SQLx
├── server/               # Entry point Axum: pool, rotas e static assets (sem SQL)
├── migrations/           # SQLx
├── .sqlx/                # metadados de queries para build offline
└── public/               # PWA manifest + service worker (a criar)
```

Regra de ouro: toda query SQL fica em `app/src/api/` dentro de `#[server]`. O `server/` é apenas o entry point Axum; o frontend nunca vê SQL. Cálculo é sempre delegado a `domain/`.

## 6. Comandos de verificação

```bash
cargo test -p domain
cargo clippy -p domain -- -D warnings
cargo build -p domain
cargo fmt --all -- --check

# anti-stub
grep -rnE "todo!|unimplemented!|dbg!" domain/src/ || echo "OK: sem stubs"

# anti-escopo e anti-dependência
grep -rniE "animal|reproducao|genealogia" domain/src/ || echo "OK: escopo limpo"
grep -rniE "leptos|axum|sqlx|tokio|reqwest" domain/Cargo.toml || echo "OK: domínio puro"
```

Quando a UI existir, adicionar `cargo leptos watch` e o teste com produtor real.

## 7. Estado atual

- **Ciclo 1:** aprovado e tagueado (`ciclo-1-aprovado`); 27 testes passando, `clippy -D warnings` limpo.
- **Ciclo 2:** aprovado e tagueado (`ciclo-2-aprovado`); PostgreSQL 16 via Docker, migrações SQLx, 7 server functions em `app/src/api/`, `.sqlx/` versionado e scaffold Leptos/Axum funcionando com `cargo leptos watch`.
- **Ciclo 3:** não iniciado. A próxima fatia é a UI do termômetro.
- O crate `domain/` continua congelado; alterações só com contrato que autorize explicitamente.

## 8. Ordem dos ciclos

| Ciclo | Fatia | Critério de pronto |
| :--- | :--- | :--- |
| 1 | `domain/` puro com cálculos e testes | `cargo test -p domain` verde; contrato auditado |
| 2 | Migrações + server functions + `resumo_lote` | Criar lote, 2 pesagens e 1 custo; resumo correto |
| 3 | Lista de lotes + detalhe com termômetro | Produtor entende em menos de 10 segundos |
| 4 | Preço real + sparkline + alerta de margem | Tela mostra hoje vs. 7 dias e aviso no limiar |
| 5 | Registro por áudio → LLM → gravação | Testado com um produtor real |

Não começar o próximo ciclo sem o anterior aprovado.

## 9. Definition of Done de um ciclo

- Todos os CAs do contrato satisfeitos e citáveis por `arquivo:linha`.
- Comandos de verificação executados com saída registrada.
- Nenhum stub, dependência proibida ou feature fora de escopo.
- Veredito do auditor = APROVADO.
- Validação humana (testes + comportamento) concluída.
- Aprendizados incorporados neste `AGENTS.md` antes do próximo ciclo.

## 10. Atualização deste documento

Ao final de cada ciclo aprovado, atualizar:

- seção 7 (estado atual);
- decisões técnicas que mudaram;
- riscos e aprendizados que devem influenciar o próximo contrato.

Não usar este arquivo para justificar expansão de escopo. A seção 3 é um limite, não um cardápio.
