use std::{collections::HashMap, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
    response::Html,
};

use shared::{AppError, SharedState};

use minijinja::{path_loader, Environment, Value};


pub async fn index() -> &'static str {
    "Welcome!"
}

async fn index2(env: Arc<Environment<'static>>) -> Html<String> {
    let mut ctx = HashMap::new();
    ctx.insert("title".to_string(), Value::from("Minha Página Incrível"));
    ctx.insert("user".to_string(), Value::from("João"));

    let tmpl = env.get_template("index.html").unwrap();
    let rendered = tmpl.render(Value::from(ctx)).unwrap();

    Html(rendered)
}