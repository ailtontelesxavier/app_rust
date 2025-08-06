use anyhow::Result;
use tokio;

#[tokio::test]
async fn test_login() {
    assert_eq!(2 + 2, 4);
}

#[tokio::test]
async fn test_hello_world() -> httpc_test::Result<()> {
    let hc = httpc_test::new_client("http://localhost:2000")?;

    let res = hc.do_get("/hello").await?; // httpc_test::Response 
    let status = res.status();
    // Pretty print the result (status, headers, response cookies, client cookies, body)
    res.print().await?;

    let body = res.text_body()?;
    assert_eq!(body, "Welcome!");

    assert_eq!(status, 200);
    Ok(())
}
