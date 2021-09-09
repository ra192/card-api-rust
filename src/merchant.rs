use crate::db::{DBPool, get_db_conn};
use crate::Errors::MerchantError;
use crate::Errors;

pub struct Merchant {
    pub id: i32,
    pub name: String,
    pub secret: String,
}

pub async fn get_merchant_by_id(pool: &DBPool, id: i32) -> Result<Merchant, Errors> {
    let conn = get_db_conn(pool).await;
    match conn.query("select name, secret from merchant where id=$1", &[&id]).await
        .map_err(|e| {
            MerchantError(e.to_string())
        })?.get(0) {
        None => {
            Err(MerchantError("merchant does not exist".to_string()))
        }
        Some(row) => {
            Ok(Merchant {
                id,
                name: row.get("name"),
                secret: row.get("secret"),
            })
        }
    }
}
