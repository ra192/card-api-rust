use crate::db::{DBPool, get_db_conn, DBConn};
use warp::reply::{Json, json};
use crate::merchant::get_merchant_by_id;
use hmac::{Hmac, NewMac};
use sha2::{Sha256, Digest};
use std::collections::BTreeMap;
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Serialize, Deserialize};
use crate::{ErrorResponse, Errors};
use crate::Errors::MerchantError;

const SECRET: &[u8; 44] = b"UCnmDHn9QS+GqLR5Gkyfw00fykPgW8R9b9uALi4xHEA=";

#[derive(Deserialize)]
pub struct TokenRequest {
    #[serde(rename = "merchantId")]
    pub merchant_id: i32,
    pub secret: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

pub async fn create_token_handler(db_pool: DBPool, req: TokenRequest) -> Result<Json, warp::Rejection> {
    let conn = get_db_conn(&db_pool).await;
    match create_token(&conn, req.merchant_id, &req.secret).await {
        Ok(token) => {
            Ok(json(&TokenResponse {
                token
            }))
        }
        Err(MerchantError(message)) => { Ok(json(&ErrorResponse { error: message })) }
        _ => { Ok(json(&ErrorResponse { error: "General error".to_string() })) }
    }
}

async fn create_token(conn: &DBConn, merchant_id: i32, secret: &String) -> Result<String, Errors> {
    let merchant = get_merchant_by_id(conn, merchant_id).await?;
    if merchant.secret != sha256_hash(secret) {
        return Err(Errors::MerchantError("Secret is not valid".to_string()));
    }
    let key: Hmac<Sha256> = Hmac::new_from_slice(SECRET).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", merchant_id.to_string());

    Ok(claims.sign_with_key(&key).unwrap())
}

pub fn validate_auth_header(auth: String) -> i32 {
    validate_token(auth.replace("Bearer", "").trim())
}

fn validate_token(token: &str) -> i32 {
    let key: Hmac<Sha256> = Hmac::new_from_slice(SECRET).unwrap();
    let claims: BTreeMap<String, String> = token.verify_with_key(&key).unwrap();
    claims["sub"].parse().unwrap()
}

fn sha256_hash(text: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    base64::encode(hasher.finalize())
}