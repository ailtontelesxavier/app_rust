use rstest::fixture;
use rstest::rstest;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use tracing::info;

use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;


#[fixture]
pub async fn postgres_pool() -> Pool<Postgres> {

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    


    // Use a imagem oficial do PostgreSQL com configuração mais simples
    let image: testcontainers::ContainerRequest<GenericImage> = GenericImage::new("postgres", "16") // alpine é mais leve
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(5432))
        .with_env_var("POSTGRES_USER", "postgres") // usuário padrão
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "test_db");

    println!("iniciando container");
    let container = image.start().await.unwrap();
    println!("iniciado container");

    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string = format!("postgres://postgres:password@localhost:{}/test_db", port);

    // Aguarda um pouco mais
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("iniciando conecção");
    let pool: Pool<Postgres> =   PgPoolOptions::new()
        .max_connections(5) // Número menor de conexões para testes
        .connect(&connection_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Successfully connected to PostgreSQL on port {}", port);

    // Testa uma conexão simples antes de aplicar migrations
    let test_result: Result<(i32,), _> = sqlx::query_as("SELECT 1").fetch_one(&pool).await;

    if let Err(e) = test_result {
        eprintln!("Initial test query failed: {}", e);
        panic!("Database not ready: {}", e);
    }

    // Aplica migrations (com tratamento de erro)
    match sqlx::migrate!("../migrations").run(&pool).await {
        Ok(_) => println!("Migrations applied successfully"),
        Err(e) => {
            eprintln!("Warning: Migrations failed: {}", e);
            // Continua mesmo se as migrations falharem
        }
    }

    pool
}

#[rstest]
#[tokio::test]
async fn test_select_1(#[future] postgres_pool: Pool<Postgres>) {
    let _ = tracing_subscriber::fmt::try_init(); // Inicializa logging
    let pool = postgres_pool.await;
    println!("Database configured successfully");

    // Abordagem direta - obtém uma conexão do pool explicitamente
    let mut connection = pool.acquire().await.expect("Failed to acquire connection");

    //let mut connection = pool.acquire().await.expect("Failed to acquire connection");
    println!("conectando novamente");
    
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Query failed");
    println!("conectado");

    println!("{:?}", row);

    // Libera a conexão explicitamente

    drop(connection);

    //drop(connection);
    assert_eq!(row.0, 1);
    println!("Test completed successfully");
}
//cargo test --package app --test db_test
