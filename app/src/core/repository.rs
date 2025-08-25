use reqwest::Client;
use sqlx::PgPool;
use tracing::debug;

use crate::core::schema::{MunicipioIbge, UfIbge};

/// Busca todas as UFs na API do IBGE
pub async fn fetch_ufs() -> Result<Vec<UfIbge>, reqwest::Error> {
    let url = "https://servicodados.ibge.gov.br/api/v1/localidades/estados";
    let resp = Client::new().get(url).send().await?;
    let ufs: Vec<UfIbge> = resp.json().await?;
    Ok(ufs)
}

/// Busca municípios de uma UF específica (ex: "TO")
pub async fn fetch_municipios(sigla: &str) -> Result<Vec<MunicipioIbge>, reqwest::Error> {
    let url = format!(
        "https://servicodados.ibge.gov.br/api/v1/localidades/estados/{}/municipios",
        sigla
    );
    debug!("Fetching municipios for UF {}: {}", sigla, url);
    let resp = Client::new().get(&url).send().await.unwrap();
    let municipios: Vec<MunicipioIbge> = resp.json().await.unwrap();
    debug!("Fetched {} municipios for UF {}", municipios.len(), sigla);
    Ok(municipios)
}

/// Salva ou atualiza UFs no banco
pub async fn upsert_ufs(pool: &PgPool, ufs: &[UfIbge]) -> Result<(), sqlx::Error> {
    for uf in ufs {
        sqlx::query!(
            r#"
            INSERT INTO uf (id, sigla, nome)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET sigla = EXCLUDED.sigla, nome = EXCLUDED.nome
            "#,
            uf.id,
            uf.sigla,
            uf.nome
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Salva ou atualiza Municípios no banco
pub async fn upsert_municipios(
    pool: &PgPool,
    municipios: Vec<MunicipioIbge>,
    uf_id: &i64,
) -> Result<(), sqlx::Error> {
    for m in municipios {
        sqlx::query!(
            r#"
            INSERT INTO municipio (id, nome, uf_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET nome = EXCLUDED.nome, uf_id = EXCLUDED.uf_id
            "#,
            m.id,
            m.nome,
            uf_id
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}
