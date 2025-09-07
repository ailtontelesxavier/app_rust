use rstest::fixture;
use rstest::rstest;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use testcontainers::ImageExt;

#[fixture]
pub async fn postgres_pool() -> Pool<Postgres> {
    // Configura a imagem do PostgreSQL
    let image = GenericImage::new("postgres", "16")
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(5438))
        .with_exposed_port(testcontainers::core::ContainerPort::Udp(5438))
        .with_exposed_port(testcontainers::core::ContainerPort::Sctp(5438))
        .with_env_var("POSTGRES_USER", "user")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "test_db");

    let container = image.start().await.unwrap();

    // Obtém a porta mapeada do host
    let port = container.get_host_port_ipv4(5438).await.unwrap();
    let connection_string =
        format!("postgres://user:password@127.0.0.1:{}/test_db", port);

    // Cria o pool de conexões com retry para aguardar o banco ficar pronto
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .unwrap();

    // Aplica migrations (se necessário)
    // Certifique-se de que o caminho das migrations está correto
    sqlx::migrate!("../migrations") // Ajuste o caminho conforme sua estrutura
        .run(&pool)
        .await
        .unwrap();

    pool
}

#[rstest]
#[tokio::test]
async fn test_select_1(#[future] postgres_pool: Pool<Postgres>) {
    let pool = postgres_pool.await;
    
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(row.0, 1);
}