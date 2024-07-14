use crate::domain::errors::LogicError;
use lambda_http::{Error, Request, RequestExt};

#[derive(serde::Deserialize)]
struct Authorizer {
    lambda: AuthorizerLambda,
}

#[derive(serde::Deserialize)]
struct AuthorizerLambda {
    claims: AuthorizerClaims,
}

#[derive(serde::Deserialize)]
pub struct AuthorizerClaims {
    pub email: String,
    #[serde(rename = "cognito:username")]
    pub username: String,
}

pub fn get_auth_claims(event: &Request) -> Result<AuthorizerClaims, Error> {
    let ctx = event.request_context().clone();
    let ctx_str = serde_json::to_string(&ctx)?;
    println!("ctx_str: {ctx_str}");

    let auth = ctx.authorizer().ok_or(LogicError::NotAllowed)?;
    let auth_str = serde_json::to_string(&auth)?;
    println!("auth_str: {auth_str}");

    let authorizer: Authorizer = serde_json::from_str(&auth_str)?;
    Ok(authorizer.lambda.claims)
}
