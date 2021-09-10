use warp::reply::{Json, json};
use serde::{Serialize, Deserialize};
use crate::token::validate_auth_header;
use crate::db::{DBPool, get_db_conn, DBConn};
use crate::{account, ErrorResponse, Errors};
use crate::Errors::{TransactionError, AccountError};

pub enum TransactionType {
    Fund
}

impl TransactionType {
    fn to_db_val(&self) -> &'static str {
        match self {
            TransactionType::Fund => { "fund" }
        }
    }
}

pub enum TransactionStatus {
    Completed
}

impl TransactionStatus {
    fn to_db_val(&self) -> &'static str {
        match self {
            TransactionStatus::Completed => { "completed" }
        }
    }
}

#[derive(Deserialize)]
pub struct FundRequest {
    #[serde(rename = "accountId")]
    pub account_id: i32,
    pub amount: i32,
    #[serde(rename = "orderId")]
    pub order_id: String,
}

#[derive(Serialize)]
pub struct FundResponse {
    pub transaction_id: i32,
}

pub async fn fund_account_handler(pool: DBPool, auth: String, req: FundRequest) -> Result<Json, warp::Rejection> {
    validate_auth_header(auth);
    let conn=get_db_conn(&pool).await;
    match fund(&conn, req).await {
        Ok(id) => {
            Ok(json(&FundResponse {
                transaction_id: id
            }))
        }
        Err(AccountError(message)) => {
            Ok(json(&ErrorResponse { error: message }))
        }
        Err(TransactionError(message)) => {
            Ok(json(&ErrorResponse { error: message }))
        }
        _ => {
            Ok(json(&ErrorResponse { error: "general error".to_string() }))
        }
    }
}

pub async fn fund(conn: &DBConn, req: FundRequest) -> Result<i32, Errors> {
    create(conn, account::CASH_ACCOUNT_ID, req.account_id, req.amount,
           TransactionType::Fund, req.order_id).await
}

pub async fn create(conn: &DBConn, src_account_id: i32, dest_account_id: i32, amount: i32,
                    trans_type: TransactionType, order_id: String) -> Result<i32, Errors> {
    let src_account = account::get_active_by_id(conn, src_account_id).await?;
    let dest_account = account::get_active_by_id(conn, dest_account_id).await?;

    if src_account.currency != dest_account.currency {
        return Err(
            TransactionError("source account currency doesn't match destination account currency".to_string()));
    }

    let trans_id: i32 = conn.query(
        "insert into transaction (id,type,status,order_id) values(default,$1,$2,$3) returning id",
        &[&trans_type.to_db_val(), &TransactionStatus::Completed.to_db_val(), &order_id]).await
        .map_err(|e| {
            TransactionError(e.to_string())
        })?.get(0).unwrap().get("id");

    conn.execute(
        "insert into transaction_item (id, amount, now(), trans_id, src_acc_id, dest_acc_id) values (default, $1, $2, $3, $4)",
        &[&amount, &trans_id, &src_account.id, &dest_account.id]).await
        .map_err(|e| {
            TransactionError(e.to_string())
        })?;

    info!("transaction with type: {} was created",trans_type.to_db_val());

    Ok(trans_id)
}