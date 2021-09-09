use crate::db::DBConn;
use crate::Errors::MerchantError;
use crate::Errors;

pub struct Merchant {
    pub id: i32,
    pub name: String,
    pub secret: String,
}

pub async fn get_merchant_by_id(conn: &DBConn, id: i32) -> Result<Merchant, Errors> {
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
