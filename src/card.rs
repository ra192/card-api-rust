use crate::db::{DBPool, DBConn, get_db_conn};
use crate::token::validate_auth_header;
use serde::{Serialize, Deserialize};
use crate::Errors::CardError;
use crate::{Errors, ErrorResponse};
use warp::reply::{Json, json};
use warp::Rejection;

#[derive(Deserialize)]
pub struct CreateRequest {
    #[serde(rename = "customerId")]
    pub customer_id: i32,
    #[serde(rename = "accountId")]
    pub account_id: i32,
}

#[derive(Serialize)]
pub struct CreateResponse {
    pub card_id: i32,
}

pub async fn create_virtual_handler(pool: DBPool, auth: String, req: CreateRequest) -> Result<Json, Rejection> {
    validate_auth_header(auth);
    let conn = get_db_conn(&pool).await;
    match create(&conn, req).await {
        Ok(id) => {
            Ok(json(&CreateResponse {
                card_id: id
            }))
        }
        Err(CardError(message)) => {
            Ok(json(&ErrorResponse {
                error: message
            }))
        }
        _ => {
            Ok(json(&ErrorResponse {
                error: "general error".to_string()
            }))
        }
    }
}

pub async fn create(conn: &DBConn, req: CreateRequest) -> Result<i32, Errors> {
    let id: i32 = conn.query("insert into card (id, type, created, cust_id, acc_id)\
     values (default, 'virtual', now(), $1, $2) returning id", &[&req.customer_id, &req.account_id]).await
        .map_err(|e| {
            CardError(e.to_string())
        })?.get(0).unwrap().get("id");
    info!("customer was created with id: {}",id);
    Ok(id)
}