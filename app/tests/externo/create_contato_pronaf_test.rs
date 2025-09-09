use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::{Router, routing::post};
use hyper::header::{CONTENT_TYPE, LOCATION};
use std::fs;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_create_contato_pronaf() {
    // Aqui você deve montar sua app real, com as rotas e estado necessários
    let app = Router::new()
        // .route("/externo/contato-form-pronaf", post(create_contato_pronaf))
        // Adapte para usar sua função real
        ;

    // Monta o corpo multipart manualmente
    let boundary = "X-BOUNDARY";
    let mut body = Vec::new();
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(b"Content-Disposition: form-data; name=\"cpf_cnpj\"\r\n\r\n12345678901\r\n");
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(b"Content-Disposition: form-data; name=\"nome\"\r\n\r\nFulano de Tal\r\n");
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(b"Content-Disposition: form-data; name=\"telefone\"\r\n\r\n63999999999\r\n");
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(b"Content-Disposition: form-data; name=\"email\"\r\n\r\nfulano@email.com\r\n");
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(b"Content-Disposition: form-data; name=\"cidade_id\"\r\n\r\n1\r\n");
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(b"Content-Disposition: form-data; name=\"val_solicitado\"\r\n\r\n1000.00\r\n");
    // Exemplo de arquivo
    let file_path = "tests/data/teste.jpg";
    if let Ok(file_bytes) = fs::read(file_path) {
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        body.extend(b"Content-Disposition: form-data; name=\"documento\"; filename=\"teste.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n");
        body.extend(&file_bytes);
        body.extend(b"\r\n");
    }
    body.extend(format!("--{}--\r\n", boundary).as_bytes());

    let req = Request::builder()
        .method("POST")
        .uri("/externo/contato-form-pronaf")
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::FOUND); // 302
    let headers = response.headers();
    let location = headers
        .get(LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(location.contains("/externo/contato-form/"));
}
