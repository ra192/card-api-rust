mod db;
mod token;
mod merchant;
mod transaction;
mod account;
mod card;
mod customer;

use warp::Filter;
use crate::db::{create_pool, DBPool};
use std::convert::Infallible;
use serde::{Serialize};

fn with_db(db_pool: DBPool) -> impl Filter<Extract=(DBPool, ), Error=Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

pub enum Errors {
    MerchantError(String),
    AccountError(String),
    TransactionError(String)
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[tokio::main]
async fn main() {
    let pool = create_pool().unwrap();

    let token_route = warp::path!("api"/"token").and(warp::post())
        .and(with_db(pool.clone())).and(warp::body::json())
        .and_then(token::create_token_handler);

    let fund_route = warp::path!("api"/"account"/"fund").and(warp::post())
        .and(with_db(pool.clone())).and(warp::header("Authorization"))
        .and(warp::body::json()).and_then(transaction::fund_account_handler);

    let routes=token_route.or(fund_route);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}