use warp::reply::{Json, json};
use serde::{Serialize, Deserialize};
use crate::token::validate_auth_header;
use crate::db::{DBPool, get_db_conn, DBConn};
use crate::{account, ErrorResponse, Errors};
use crate::Errors::{TransactionError, AccountError};
use tokio_postgres::Row;

pub enum TransactionType {
    Fund,
    VirtualCardDeposit,
    VirtualCardWithdraw,
}

impl TransactionType {
    fn to_db_val(&self) -> &'static str {
        match self {
            TransactionType::Fund => { "fund" }
            TransactionType::VirtualCardDeposit => { "virtual_card_deposit" }
            TransactionType::VirtualCardWithdraw => { "virtual_card_withdraw" }
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
    let conn = get_db_conn(&pool).await;
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

pub async fn deposit(conn: &DBConn, src_account_id: i32, dest_account_id: i32, fee_account_id: i32, amount: i32,
                     trans_type: TransactionType, order_id: String) -> Result<i32, Errors> {
    let fee = calculate_fee(conn, amount, &trans_type, dest_account_id).await?;

    if get_sum(conn, src_account_id).await? - (amount as i64) < 0 {
        return Err(TransactionError("source account does not have enough funds".to_string()));
    }

    let trans_id = create(conn, src_account_id, dest_account_id, amount, &trans_type, order_id).await?;
    if fee > 0 {
        create_item(conn, fee, trans_id, dest_account_id, fee_account_id).await?;
    }

    info!("transaction with type: {} was created",trans_type.to_db_val());

    Ok(trans_id)
}

pub async fn fund(conn: &DBConn, req: FundRequest) -> Result<i32, Errors> {
    let trans_id = create(conn, account::CASH_ACCOUNT_ID, req.account_id, req.amount,
                          &TransactionType::Fund, req.order_id).await?;
    info!("transaction with type: {} was created",TransactionType::Fund.to_db_val());
    Ok(trans_id)
}

pub async fn withdraw(conn: &DBConn, src_account_id: i32, dest_account_id: i32, fee_account_id: i32, amount: i32,
                      trans_type: TransactionType, order_id: String) -> Result<i32, Errors> {
    let fee = calculate_fee(conn, amount, &trans_type, dest_account_id).await?;

    if get_sum(conn, src_account_id).await? - (amount as i64) - (fee as i64) < 0 {
        return Err(TransactionError("source account does not have enough funds".to_string()));
    }

    let trans_id = create(conn, src_account_id, dest_account_id, amount, &trans_type, order_id).await?;
    if fee > 0 {
        create_item(conn, fee, trans_id, src_account_id, fee_account_id).await?;
    }

    info!("transaction with type: {} was created",trans_type.to_db_val());

    Ok(trans_id)
}

async fn create(conn: &DBConn, src_account_id: i32, dest_account_id: i32, amount: i32,
                trans_type: &TransactionType, order_id: String) -> Result<i32, Errors> {
    let src_account = account::get_active_by_id(conn, src_account_id).await?;
    let dest_account = account::get_active_by_id(conn, dest_account_id).await?;

    if src_account.currency != dest_account.currency {
        return Err(
            TransactionError("source account currency doesn't match destination account currency".to_string()));
    }

    let trans_id: i32 = conn.query(
        "insert into transaction (id,type,status,order_id) values (default,$1,$2,$3) returning id",
        &[&trans_type.to_db_val(), &TransactionStatus::Completed.to_db_val(), &order_id]).await
        .map_err(|e| {
            TransactionError(e.to_string())
        })?.get(0).unwrap().get("id");

    create_item(conn, amount, trans_id, src_account_id, dest_account_id).await?;

    Ok(trans_id)
}

async fn create_item(conn: &DBConn, amount: i32, trans_id: i32, src_account_id: i32, dest_acccount_id: i32) -> Result<u64, Errors> {
    Ok(conn.execute(
        "insert into transaction_item (id, amount, created, trans_id, src_acc_id, dest_acc_id) values(default, $1, now(), $2, $3, $4)",
        &[&amount, &trans_id, &src_account_id, &dest_acccount_id]).await
        .map_err(|e| {
            TransactionError(e.to_string())
        })?)
}

async fn get_sum(conn: &DBConn, account_id: i32) -> Result<i64, Errors> {
    Ok(get_sum_by_dest_acc(conn, account_id).await? - get_sum_by_src_acc(conn, account_id).await?)
}

async fn get_sum_by_src_acc(conn: &DBConn, account_id: i32) -> Result<i64, Errors> {
    conn.query("select sum(amount) from transaction_item where src_acc_id=$1", &[&account_id]).await
        .map_err(|e| {
            TransactionError(e.to_string())
        })?.get(0).map(get_sum_from_row).unwrap()
}

async fn get_sum_by_dest_acc(conn: &DBConn, account_id: i32) -> Result<i64, Errors> {
    conn.query("select sum(amount) from transaction_item where dest_acc_id=$1", &[&account_id]).await
        .map_err(|e| {
            TransactionError(e.to_string())
        })?.get(0).map(get_sum_from_row).unwrap()
}

fn get_sum_from_row(row: &Row) -> Result<i64, Errors> {
    let sum_opt: Option<i64>=row.get(0);
    match sum_opt {
        None => {Ok(0)}
        Some(sum) => {Ok(sum)}
    }
}

async fn calculate_fee(conn: &DBConn, amount: i32, trans_type: &TransactionType, account_id: i32) -> Result<i32, Errors> {
    conn.query("select rate from transaction_fee where type = $1 and acc_id = $2",
               &[&trans_type.to_db_val(), &account_id]).await
        .map_err(|e| {
            TransactionError(e.to_string())
        })?.get(0).map(|row| {
        let rate: f32 = row.get("rate");
        let amount_f: f32 = amount as f32;
        let res: i32 = (rate * amount_f) as i32;
        Ok(res)
    }).unwrap_or(Ok(0))
}