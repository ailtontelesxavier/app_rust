use std::{collections::HashMap, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
    response::Html,
};

use shared::{AppError, SharedState};

use minijinja::{path_loader, Environment, Value, context};


pub async fn index(env: Arc<Environment<'static>>) -> Html<String> {

    println!("Current dir: {:?}", std::env::current_dir().unwrap());

    let tmpl = env.get_template("index.html").expect("template not found");

    let rendered = tmpl.render(context! {}).unwrap();

   Html(rendered)
}

async fn index2(env: Arc<Environment<'static>>) -> Html<String> {
    let mut ctx = HashMap::new();
    ctx.insert("title".to_string(), Value::from("Minha Página Incrível"));
    ctx.insert("user".to_string(), Value::from("João"));

    let tmpl = env.get_template("index.html").unwrap();
    let rendered = tmpl.render(Value::from(ctx)).unwrap();

    Html(rendered)
}