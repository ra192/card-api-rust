use chrono::prelude::*;
use crate::db::{DBPool, get_db_conn, DBConn};
use crate::token::validate_auth_header;
use crate::Errors::CustomerError;
use crate::Errors;

pub struct Customer {
    pub id: i32,
    pub phone: String,
    pub email: String,
    pub active: bool,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: NaiveDate,
    pub address: String,
    pub address2: String,
    pub city: String,
    pub state_region: String,
    pub country: String,
    pub postal_code: String,
    pub merchant_id: i32,
}

pub struct CreateRequest {
    pub phone: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: NaiveDate,
    pub address: String,
    pub address2: String,
    pub city: String,
    pub state_region: String,
    pub country: String,
    pub postal_code: String,
}

pub async fn create_handler(pool: DBPool, auth: String, req: CreateRequest) {
    let merchant_id = validate_auth_header(auth);
    let conn = get_db_conn(&pool).await;
    create(&conn, req, merchant_id).await;
}

pub async fn create(conn: &DBConn, req: CreateRequest, merch_id: i32) -> Result<i32, Errors> {
    let id:i32=conn.query("insert into customer\
     (id, phone, email, active, first_name, last_name, birth_date, address, address2, city, state_region, country, postal_code, merch_id) values\
       (default, $1, $2, true, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) returning id",
               &[&req.phone, &req.email, &req.first_name, &req.last_name, &req.birth_date,
                   &req.address, &req.address2, &req.city, &req.state_region, &req.country, &req.postal_code, &merch_id]).await
        .map_err(|e| {
            CustomerError(e.to_string())
        })?.get(0).unwrap().get("id");
    Ok(id)
}

