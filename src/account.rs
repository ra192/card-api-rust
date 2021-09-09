use crate::db::{DBPool, get_db_conn};
use crate::Errors;
use crate::Errors::AccountError;

pub const CASH_ACCOUNT_ID: i32 = 1;

pub struct Account {
    pub id: i32,
    pub name: String,
    pub active: bool,
    pub currency: String,
    pub merch_id: i32,
}

pub async fn get_active_by_id(pool: &DBPool, id: i32) -> Result<Account, Errors> {
    let conn = get_db_conn(pool).await;
    match conn.query("select * from account where id=$1 and active = true", &[&id]).await
        .map_err(|e| {
            AccountError(e.to_string())
        })?.get(0) {
        None => {
            Err(AccountError("account does not exist".to_string()))
        }
        Some(row) => {
            Ok(Account {
                id,
                name: row.get("name"),
                active: true,
                currency: row.get("currency"),
                merch_id: row.get("merch_id"),
            })
        }
    }
}