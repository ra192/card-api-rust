use chrono::prelude::*;
use crate::db::{DBPool, get_db_conn};

pub struct Customer {
    pub id: i32,
    pub phone: String,
    pub email: String,
    pub active: bool,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Date<Local>,
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
    pub birth_date: Date<Local>,
    pub address: String,
    pub address2: String,
    pub city: String,
    pub state_region: String,
    pub country: String,
    pub postal_code: String,
}

pub async fn create_handler(pool: DBPool, auth: String, req: CreateRequest) {}

pub async fn create(pool: &DBPool) {
    let conn = get_db_conn(pool).await;
    conn.query("insert into customer\
     (id, phone, email, active, first_name, last_name, birth_date, address, address2, city, state_region, country, postal_code, merch_id) values\
       (default, $1, $2, true, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) returning id",
               &[]);
}

