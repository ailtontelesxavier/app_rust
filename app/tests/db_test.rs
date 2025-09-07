use rstest::fixture;
use tracing::info;
use rstest::rstest;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use std::time::Duration;

#[fixture]
pub async fn postgres_pool() -> Pool<Postgres> {
    // Use a imagem oficial do PostgreSQL com configuração mais simples
    let image = GenericImage::new("postgres", "16-alpine") // alpine é mais leve
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(5432))
        .with_env_var("POSTGRES_USER", "postgres") // usuário padrão
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "test_db");

    let container = image.start().await.unwrap();

    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string = format!(
        "postgres://postgres:password@localhost:{}/test_db",
        port
    );

    // Aguarda um pouco mais
    tokio::time::sleep(Duration::from_secs(3)).await;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Successfully connected to PostgreSQL on port {}", port);

    // Aplica migrations (se necessário)
    // Certifique-se de que o caminho das migrations está correto
    sqlx::migrate!("../migrations") // Ajuste o caminho conforme sua estrutura
        .run(&pool)
        .await
        .unwrap();
    println!("aplicado migracoes");
    
    pool
}

#[rstest]
#[tokio::test]
async fn test_select_1(#[future] postgres_pool: Pool<Postgres>) {
    let _ = tracing_subscriber::fmt::try_init(); // Inicializa logging
    let pool = postgres_pool.await;

    println!("db configurando com sucesso");
    
    // Teste muito simples primeiro
    let result = sqlx::query("SELECT 1")
        .execute(&pool)
        .await;

    println!("query teste");
    
    assert!(result.is_ok(), "Basic query failed: {:?}", result.err());
    
    // Depois o teste original
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(row.0, 1);
}