use chrono::prelude::*;

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
}

pub async fn create() {

}