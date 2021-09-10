use chrono::prelude::*;
use crate::db::{DBPool, get_db_conn, DBConn};
use crate::token::validate_auth_header;
use crate::Errors::CustomerError;
use crate::{Errors, ErrorResponse};
use serde::{Serialize, Deserialize};
use warp::reply::{Json, json};
use warp::Rejection;

#[derive(Deserialize)]
pub struct CreateRequest {
    pub phone: String,
    pub email: String,
    #[serde(rename="firstName")]
    pub first_name: String,
    #[serde(rename="lastName")]
    pub last_name: String,
    #[serde(rename="birthDate")]
    pub birth_date: String,
    pub address: String,
    pub city: String,
    #[serde(rename="stateRegion")]
    pub state_region: String,
    pub country: String,
    #[serde(rename="postalCode")]
    pub postal_code: String,
}

#[derive(Serialize)]
pub struct CreateResponse {
    pub customer_id: i32,
}

pub async fn create_handler(pool: DBPool, auth: String, req: CreateRequest) -> Result<Json, Rejection> {
    let merchant_id = validate_auth_header(auth);
    let conn = get_db_conn(&pool).await;
    match create(&conn, req, merchant_id).await {
        Ok(id) => {
            Ok(json(&CreateResponse {
                customer_id: id
            }))
        }
        Err(CustomerError(message)) => {
            Ok(json(&ErrorResponse {
                error: message
            }))
        }
        _ => { Ok(json(&ErrorResponse { error: "general error".to_string() })) }
    }
}

pub async fn create(conn: &DBConn, req: CreateRequest, merch_id: i32) -> Result<i32, Errors> {
    let birth_date=NaiveDate::parse_from_str(&req.birth_date,"%Y-%m-%d").map_err(|_| {
        CustomerError("birthDate is not valid".to_string())
    })?;

    let id: i32 = conn.query("insert into customer\
     (id, phone, email, active, first_name, last_name, birth_date, address, city, state_region, country, postal_code, merch_id) values\
       (default, $1, $2, true, $3, $4, $5, $6, $7, $8, $9, $10, $11) returning id",
                             &[&req.phone, &req.email, &req.first_name, &req.last_name, &birth_date,
                                 &req.address, &req.city, &req.state_region, &req.country, &req.postal_code, &merch_id]).await
        .map_err(|e| {
            CustomerError(e.to_string())
        })?.get(0).unwrap().get("id");
    info!("customer was created with id: {}", id);
    Ok(id)
}

