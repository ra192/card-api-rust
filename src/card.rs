use crate::db::{DBPool, DBConn, get_db_conn};
use crate::token::validate_auth_header;
use serde::{Serialize, Deserialize};
use crate::Errors::{CardError, TransactionError};
use crate::{Errors, ErrorResponse};
use warp::reply::{Json, json};
use warp::Rejection;
use crate::transaction;
use chrono::prelude::*;
use crate::transaction::TransactionType::{VirtualCardDeposit, VirtualCardWithdraw};

const CARD_ACCOUNT_ID: i32 = 2;
const FEE_ACCOUNT_ID: i32 = 3;

pub struct Card {
    pub id: i32,
    pub card_type: String,
    pub created: DateTime<Local>,
    pub acc_id: i32,
    pub cust_id: i32,
}

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

#[derive(Deserialize)]
pub struct TransactionRequest {
    #[serde(rename = "cardId")]
    pub card_id: i32,
    pub amount: i32,
    #[serde(rename = "orderId")]
    pub order_id: String,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub trans_id: i32,
}

pub async fn deposit_virtual_handler(pool: DBPool, auth: String, req: TransactionRequest) -> Result<Json, Rejection> {
    validate_auth_header(auth);
    let conn = get_db_conn(&pool).await;
    match deposit(&conn, req).await {
        Ok(id) => {
            Ok(json(&TransactionResponse {
                trans_id: id
            }))
        }
        Err(CardError(message)) => {
            Ok(json(&ErrorResponse {
                error: message
            }))
        }
        Err(TransactionError(message)) => {
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

pub async fn deposit(conn: &DBConn, req: TransactionRequest) -> Result<i32, Errors> {
    let card = get_by_id(conn, req.card_id).await?;
    transaction::withdraw(conn, card.acc_id, CARD_ACCOUNT_ID,
                          FEE_ACCOUNT_ID, req.amount, VirtualCardDeposit, req.order_id).await
}

pub async fn withdraw_virtual_handler(pool: DBPool, auth: String, req: TransactionRequest) -> Result<Json, Rejection> {
    validate_auth_header(auth);
    let conn = get_db_conn(&pool).await;
    match withdraw(&conn, req).await {
        Ok(id) => {
            Ok(json(&TransactionResponse {
                trans_id: id
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

pub async fn withdraw(conn: &DBConn, req: TransactionRequest) -> Result<i32, Errors> {
    let card = get_by_id(conn, req.card_id).await?;
    transaction::deposit(conn, card.acc_id, CARD_ACCOUNT_ID,
                         FEE_ACCOUNT_ID, req.amount, VirtualCardWithdraw, req.order_id).await
}

async fn get_by_id(conn: &DBConn, id: i32) -> Result<Card, Errors> {
    match conn.query("select * from card where id = $1", &[&id]).await.map_err(|e| {
        CardError(e.to_string())
    })?.get(0) {
        None => { Err(CardError("card does not exist".to_string())) }
        Some(row) => {
            Ok(Card {
                id,
                card_type: row.get("type"),
                created: row.get("created"),
                acc_id: row.get("acc_id"),
                cust_id: row.get("cust_id"),
            })
        }
    }
}