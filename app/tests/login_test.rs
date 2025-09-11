use anyhow::Result;
use tokio;

static BASE_URL: &str = "http://localhost:2000";

/*
Se sua API usa header Authorization: Bearer ..., troque .add_cookie("access_token", &token) por .add_header("Authorization", &format!("Bearer {}", token)).

*/
/* async fn login_and_get_token() -> httpc_test::Result<String> {
    let hc = httpc_test::new_client(BASE_URL)?;
    let res = hc
        .do_post("/login")
        .json(&serde_json::json!({
            "username": "admin",
            "password": "1234"
        }))
        .await?;

    res.print().await?;
    let status = res.status();
    assert_eq!(status, 200);

    // Pegando o token do cookie ou do corpo (ajuste conforme sua API)
    let token = res
        .cookie("access_token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            // Se retornar no corpo como JSON
            res.json_body().ok().and_then(|v: serde_json::Value| {
                v.get("access_token")
                    .and_then(|t| t.as_str().map(|s| s.to_string()))
            })
        })
        .expect("Token não encontrado!");

    Ok(token)
} */

/* #[tokio::test]
async fn test_login_and_access_private() -> httpc_test::Result<()> {
    let token = login_and_get_token().await?;

    let hc = httpc_test::new_client(BASE_URL)?;
    let res = hc
        .do_get("/privado")
        .add_cookie("access_token", &token)
        .await?;

    res.print().await?;
    assert_eq!(res.status(), 200);
    Ok(())
} */

/* #[tokio::test]
async fn test_cadastro_perfil() -> httpc_test::Result<()> {
    let token = login_and_get_token().await?;

    let hc = httpc_test::new_client(BASE_URL)?;
    let res = hc
        //.do_post("/permissao/perfil")
        //.add_cookie("access_token", &token)
        .json(&serde_json::json!({
            "nome": "Perfil Teste",
            "descricao": "Perfil criado via teste"
        }))
        .await?;

    res.print().await?;
    assert_eq!(res.status(), 200); // ou 201, conforme sua API
    Ok(())
}
 */
