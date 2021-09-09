use crate::db::DBPool;
use warp::reply::{Json, json};
use crate::merchant::get_merchant_by_id;
use hmac::{Hmac, NewMac};
use sha2::{Sha256, Digest};
use std::collections::BTreeMap;
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Serialize, Deserialize};
use crate::ErrorResponse;
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
    let merchant_res = get_merchant_by_id(&db_pool, req.merchant_id).await;
    match merchant_res {
        Ok(merchant) => {
            if merchant.secret == sha256_hash(&req.secret) {
                Ok(json(&TokenResponse {
                    token: create_token(&req.merchant_id)
                }))
            } else {
                Ok(json(&ErrorResponse { error: "Secret is not valid".to_string() }))
            }
        }
        Err(MerchantError(message)) => { Ok(json(&ErrorResponse { error:  message})) }
        _ => {Ok(json(&ErrorResponse { error:  "General error".to_string()}))}
    }
}

fn create_token(merchant_id: &i32) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(SECRET).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", merchant_id.to_string());

    claims.sign_with_key(&key).unwrap()
}

pub fn validate_auth_header(auth:String) -> i32 {
    validate_token(auth.replace("Bearer","").trim())
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