use std::collections::HashMap;

use axum::{
    body::Body, extract::Request, http::{header::CACHE_CONTROL, Method}, middleware::{self, Next}, response::{IntoResponse, Redirect, Response}, Extension, Form
};


pub async fn method_override(mut req: Request<Body>, next: Next) -> Response {
    // Verifique se é um POST
    if req.method() == Method::POST {
        // Extrai os dados do formulário
        let form_data = req.extensions().get::<Form<HashMap<String, String>>>();
        
        if let Some(form) = form_data {
            if let Some(method) = form.get("_method") {
                match method.to_uppercase().as_str() {
                    "PUT" => *req.method_mut() = Method::PUT,
                    "DELETE" => *req.method_mut() = Method::DELETE,
                    "PATCH" => *req.method_mut() = Method::PATCH,
                    _ => (),
                }
            }
        }
    }
    
    next.run(req).await
}

/*
//use tower_sessions::Session;
use crate::handlers::errors::AppError;
use crate::models::app::CurrentUser;
pub async fn authenticate(
    session: Session,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user_id = session.get::<i32>("authenticated_user_id").await?;
    
    let mut current_user = CurrentUser {
        is_authenticated: false,
        user_id: None,
    };
    
    if let Some(id) = user_id {
        current_user.is_authenticated = true;
        current_user.user_id = Some(id);
        
        req.extensions_mut().insert(current_user);
        
        Ok(next.run(req).await)
    } else {
        req.extensions_mut().insert(current_user);
        
        Ok(next.run(req).await)
    }
}

pub async fn required_authentication(
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
    next: Next,
) -> Response {
    if !current_user.is_authenticated {
        return Redirect::to("/log-in").into_response();
    }
    
    let mut res = next.run(req).await;
    
    res.headers_mut()
    .insert(CACHE_CONTROL, "no-store".parse().unwrap());

res
}

pub async fn redirect_auth_user(
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
    next: Next,
) -> Response {
    if current_user.is_authenticated {
        return Redirect::to("/todos").into_response();
    }

    next.run(req).await
}
*/