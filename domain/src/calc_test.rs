use chrono::NaiveDate;

use crate::calc::{
    calcular_custo_total, calcular_gmd, calcular_margem, calcular_peso_atual, calcular_resultado,
    calcular_valor_mercado, resumo_lote,
};
use crate::errors::DomainError;
use crate::models::{Custo, Lote, LoteStatus, Pesagem, PrecoMercado, ResumoLote, TipoCusto};

const LOTE_ID: &str = "lote-1";
const OUTRO_LOTE_ID: &str = "lote-2";

fn data(iso: &str) -> NaiveDate {
    NaiveDate::parse_from_str(iso, "%Y-%m-%d").expect("data de teste válida")
}

fn lote_base() -> Lote {
    Lote {
        id: LOTE_ID.to_string(),
        nome: "Lote 1".to_string(),
        data_entrada: data("2026-01-01"),
        qtd_animais: 100,
        peso_inicial_medio: 300.0,
        custo_compra_total: 600_000.0,
        status: LoteStatus::Ativo,
    }
}

fn pesagem(id: &str, lote_id: &str, dia: &str, peso_medio: f64) -> Pesagem {
    Pesagem {
        id: id.to_string(),
        lote_id: lote_id.to_string(),
        data: data(dia),
        peso_medio,
    }
}

fn custo(id: &str, lote_id: &str, dia: &str, tipo: TipoCusto, valor: f64) -> Custo {
    Custo {
        id: id.to_string(),
        lote_id: lote_id.to_string(),
        data: data(dia),
        tipo,
        valor,
    }
}

fn preco(preco_kg: f64) -> PrecoMercado {
    PrecoMercado {
        data: data("2026-04-11"),
        regiao: "MS".to_string(),
        preco_kg,
    }
}

#[test]
fn ca1_lote_tem_os_campos_do_contrato() {
    let lote = lote_base();

    assert_eq!(lote.id, "lote-1");
    assert_eq!(lote.nome, "Lote 1");
    assert_eq!(lote.data_entrada, data("2026-01-01"));
    assert_eq!(lote.qtd_animais, 100);
    assert_eq!(lote.peso_inicial_medio, 300.0);
    assert_eq!(lote.custo_compra_total, 600_000.0);
    assert_eq!(lote.status, LoteStatus::Ativo);
    assert_ne!(lote.status, LoteStatus::Vendido);
}

#[test]
fn ca2_pesagem_tem_os_campos_do_contrato() {
    let pesagem = pesagem("p1", LOTE_ID, "2026-04-11", 420.0);

    assert_eq!(pesagem.id, "p1");
    assert_eq!(pesagem.lote_id, "lote-1");
    assert_eq!(pesagem.data, data("2026-04-11"));
    assert_eq!(pesagem.peso_medio, 420.0);
}

#[test]
fn ca3_custo_tem_os_campos_e_tipos_do_contrato() {
    let tipos = [
        TipoCusto::Sanidade,
        TipoCusto::Suplemento,
        TipoCusto::Pastagem,
        TipoCusto::Outro,
    ];
    let custo = custo("c1", LOTE_ID, "2026-02-01", TipoCusto::Suplemento, 25_000.0);

    assert_eq!(custo.id, "c1");
    assert_eq!(custo.lote_id, "lote-1");
    assert_eq!(custo.data, data("2026-02-01"));
    assert_eq!(custo.tipo, TipoCusto::Suplemento);
    assert_eq!(custo.valor, 25_000.0);
    assert_eq!(tipos.len(), 4);
}

#[test]
fn ca4_preco_de_mercado_tem_os_campos_do_contrato() {
    let preco = preco(18.5);

    assert_eq!(preco.data, data("2026-04-11"));
    assert_eq!(preco.regiao, "MS");
    assert_eq!(preco.preco_kg, 18.5);
}

#[test]
fn ca5_resumo_lote_tem_os_campos_do_contrato() {
    let resumo = ResumoLote {
        custo_total: 633_000.0,
        gmd: Some(1.2),
        peso_atual: 420.0,
        valor_mercado: 777_000.0,
        resultado: 144_000.0,
        margem: Some(0.227_488_151_7),
        dias_no_pasto: 100,
    };

    assert_eq!(resumo.custo_total, 633_000.0);
    assert_eq!(resumo.gmd, Some(1.2));
    assert_eq!(resumo.peso_atual, 420.0);
    assert_eq!(resumo.valor_mercado, 777_000.0);
    assert_eq!(resumo.resultado, 144_000.0);
    assert!(resumo.margem.is_some());
    assert_eq!(resumo.dias_no_pasto, 100_u32);
}

#[test]
fn ca6_custo_total_soma_compra_e_custos_do_lote() {
    let lote = lote_base();
    let custos = vec![
        custo("c1", LOTE_ID, "2026-02-01", TipoCusto::Suplemento, 25_000.0),
        custo("c2", LOTE_ID, "2026-03-01", TipoCusto::Sanidade, 5_000.0),
        custo(
            "c3",
            OUTRO_LOTE_ID,
            "2026-03-01",
            TipoCusto::Pastagem,
            9_999.0,
        ),
    ];

    let total = calcular_custo_total(&lote, &custos);

    assert!((total - 630_000.0).abs() < 1e-9);
    assert_ne!(total, 639_999.0);
}

#[test]
fn ca6_custo_total_sem_custos_e_igual_ao_custo_de_compra() {
    let lote = lote_base();
    let total = calcular_custo_total(&lote, &[]);

    assert_eq!(total, 600_000.0);
}

#[test]
fn ca13_sem_pesagens_gmd_e_none_e_peso_e_o_inicial() {
    let lote = lote_base();
    let pesagens: Vec<Pesagem> = Vec::new();

    let gmd = calcular_gmd(
        lote.peso_inicial_medio,
        &pesagens,
        lote.data_entrada,
        data("2026-04-11"),
    );
    let peso_atual = calcular_peso_atual(
        lote.peso_inicial_medio,
        &pesagens,
        lote.data_entrada,
        data("2026-04-11"),
    );

    assert!(gmd.is_none());
    assert_eq!(peso_atual, 300.0);
}

#[test]
fn ca14_uma_pesagem_calcula_gmd_entre_entrada_e_pesagem() {
    let pesagens = vec![pesagem("p1", LOTE_ID, "2026-04-01", 390.0)];

    let gmd = calcular_gmd(300.0, &pesagens, data("2026-01-01"), data("2026-04-11"));

    assert!(gmd.is_some());
    assert!((gmd.unwrap_or_default() - 1.0).abs() < 1e-9);
}

#[test]
fn ca15_pesagens_fora_de_ordem_usam_a_mais_recente() {
    let pesagens = vec![
        pesagem("p1", LOTE_ID, "2026-03-01", 360.0),
        pesagem("p2", LOTE_ID, "2026-04-01", 390.0),
        pesagem("p3", LOTE_ID, "2026-02-01", 330.0),
    ];

    let gmd = calcular_gmd(300.0, &pesagens, data("2026-01-01"), data("2026-04-11"));
    let peso_atual = calcular_peso_atual(300.0, &pesagens, data("2026-01-01"), data("2026-04-11"));

    assert!(gmd.is_some());
    assert!((gmd.unwrap_or_default() - 1.0).abs() < 1e-9);
    assert_eq!(peso_atual, 390.0);
}

#[test]
fn ca15_pesagem_futura_e_ignorada() {
    let pesagens = vec![
        pesagem("p1", LOTE_ID, "2026-02-01", 330.0),
        pesagem("p2", LOTE_ID, "2026-04-01", 500.0),
    ];

    let gmd = calcular_gmd(300.0, &pesagens, data("2026-01-01"), data("2026-03-01"));
    let peso_atual = calcular_peso_atual(300.0, &pesagens, data("2026-01-01"), data("2026-03-01"));

    assert!(gmd.is_some());
    assert!((gmd.unwrap_or_default() - (30.0 / 31.0)).abs() < 1e-9);
    assert_eq!(peso_atual, 330.0);
}

#[test]
fn ca14_pesagem_no_mesmo_dia_da_entrada_nao_gera_gmd() {
    let pesagens = vec![pesagem("p1", LOTE_ID, "2026-01-01", 310.0)];

    let gmd = calcular_gmd(300.0, &pesagens, data("2026-01-01"), data("2026-04-11"));

    assert!(gmd.is_none());
}

#[test]
fn calcular_gmd_aceita_perda_de_peso() {
    let pesagens = vec![pesagem("p1", LOTE_ID, "2026-04-11", 280.0)];

    let gmd = calcular_gmd(300.0, &pesagens, data("2026-01-01"), data("2026-04-11"));

    assert!(gmd.is_some());
    assert!((gmd.unwrap_or_default() - (-0.2)).abs() < 1e-9);
}

#[test]
fn ca9_valor_de_mercado_multiplica_peso_quantidade_e_preco() {
    let valor = calcular_valor_mercado(420.0, 100, 18.5);

    assert!((valor - 777_000.0).abs() < 1e-9);
    assert!(valor > 0.0);
}

#[test]
fn ca10_resultado_positivo_representa_lucro() {
    let resultado = calcular_resultado(777_000.0, 633_000.0);

    assert!((resultado - 144_000.0).abs() < 1e-9);
    assert!(resultado > 0.0);
}

#[test]
fn ca10_resultado_negativo_representa_prejuizo() {
    let resultado = calcular_resultado(630_000.0, 633_000.0);

    assert!((resultado - (-3_000.0)).abs() < 1e-9);
    assert!(resultado < 0.0);
}

#[test]
fn ca11_margem_e_resultado_sobre_custo() {
    let margem = calcular_margem(144_000.0, 633_000.0);

    assert!(margem.is_some());
    assert!((margem.unwrap_or_default() - 0.227_488_151_658_767_8).abs() < 1e-12);
}

#[test]
fn ca16_margem_com_custo_zero_e_none() {
    let margem = calcular_margem(144_000.0, 0.0);

    assert!(margem.is_none());
    assert_ne!(margem, Some(f64::INFINITY));
}

#[test]
fn ca12_ca19_resumo_lote_calcula_o_termometro_completo() {
    let lote = lote_base();
    let pesagens = vec![pesagem("p1", LOTE_ID, "2026-04-11", 420.0)];
    let custos = vec![
        custo("c1", LOTE_ID, "2026-02-01", TipoCusto::Suplemento, 25_000.0),
        custo("c2", LOTE_ID, "2026-03-01", TipoCusto::Sanidade, 5_000.0),
        custo("c3", LOTE_ID, "2026-04-01", TipoCusto::Pastagem, 3_000.0),
    ];

    let resumo = resumo_lote(&lote, &pesagens, &custos, &preco(18.5), data("2026-04-11"))
        .expect("resumo válido");

    assert!((resumo.custo_total - 633_000.0).abs() < 1e-9);
    assert!((resumo.peso_atual - 420.0).abs() < 1e-9);
    assert!((resumo.gmd.unwrap_or_default() - 1.2).abs() < 1e-9);
    assert!((resumo.valor_mercado - 777_000.0).abs() < 1e-9);
    assert!((resumo.resultado - 144_000.0).abs() < 1e-9);
    assert!((resumo.margem.unwrap_or_default() - 0.227_488_151_658_767_8).abs() < 1e-12);
    assert_eq!(resumo.dias_no_pasto, 100_u32);
}

#[test]
fn ca12_resumo_lote_sem_pesagem_usa_peso_inicial() {
    let lote = lote_base();
    let custos = vec![custo(
        "c1",
        LOTE_ID,
        "2026-02-01",
        TipoCusto::Suplemento,
        33_000.0,
    )];

    let resumo =
        resumo_lote(&lote, &[], &custos, &preco(18.5), data("2026-04-11")).expect("resumo válido");

    assert!(resumo.gmd.is_none());
    assert_eq!(resumo.peso_atual, 300.0);
    assert!((resumo.valor_mercado - 555_000.0).abs() < 1e-9);
    assert!((resumo.resultado - (-78_000.0)).abs() < 1e-9);
    assert_eq!(resumo.dias_no_pasto, 100_u32);
}

#[test]
fn ca12_resumo_lote_ignora_pesagens_de_outro_lote() {
    let lote = lote_base();
    let pesagens = vec![
        pesagem("p1", LOTE_ID, "2026-04-01", 390.0),
        pesagem("p2", OUTRO_LOTE_ID, "2026-04-11", 500.0),
    ];

    let resumo = resumo_lote(&lote, &pesagens, &[], &preco(18.5), data("2026-04-11"))
        .expect("resumo válido");

    assert_eq!(resumo.peso_atual, 390.0);
    assert!(resumo.gmd.is_some());
}

#[test]
fn resumo_lote_ignora_custo_posterior_a_data_de_referencia() {
    let lote = lote_base();
    let custos = vec![
        custo("c1", LOTE_ID, "2026-02-01", TipoCusto::Suplemento, 10_000.0),
        custo("c2", LOTE_ID, "2026-05-01", TipoCusto::Sanidade, 99_000.0),
    ];

    let resumo =
        resumo_lote(&lote, &[], &custos, &preco(18.5), data("2026-04-11")).expect("resumo válido");

    assert!((resumo.custo_total - 610_000.0).abs() < 1e-9);
    assert_ne!(resumo.custo_total, 709_000.0);
}

#[test]
fn ca17_data_de_referencia_anterior_a_entrada_retorna_data_invalida() {
    let lote = lote_base();
    let resultado = resumo_lote(&lote, &[], &[], &preco(18.5), data("2025-12-31"));

    assert_eq!(resultado, Err(DomainError::DataInvalida));
}

#[test]
fn ca18_quantidade_zero_retorna_parametro_invalido() {
    let mut lote = lote_base();
    lote.qtd_animais = 0;

    let resultado = resumo_lote(&lote, &[], &[], &preco(18.5), data("2026-04-11"));

    assert_eq!(resultado, Err(DomainError::ParametroInvalido));
}

#[test]
fn ca18_preco_zero_retorna_parametro_invalido() {
    let lote = lote_base();

    let resultado = resumo_lote(&lote, &[], &[], &preco(0.0), data("2026-04-11"));

    assert_eq!(resultado, Err(DomainError::ParametroInvalido));
}

#[test]
fn ca18_preco_negativo_retorna_parametro_invalido() {
    let lote = lote_base();

    let resultado = resumo_lote(&lote, &[], &[], &preco(-1.0), data("2026-04-11"));

    assert_eq!(resultado, Err(DomainError::ParametroInvalido));
}

#[test]
fn ca16_resumo_sem_custos_tem_margem_none() {
    let mut lote = lote_base();
    lote.custo_compra_total = 0.0;
    let pesagens = vec![pesagem("p1", LOTE_ID, "2026-04-11", 420.0)];

    let resumo = resumo_lote(&lote, &pesagens, &[], &preco(18.5), data("2026-04-11"))
        .expect("resumo válido");

    assert_eq!(resumo.custo_total, 0.0);
    assert!(resumo.margem.is_none());
    assert!((resumo.resultado - 777_000.0).abs() < 1e-9);
}
