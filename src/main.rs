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

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::env;

fn with_db(db_pool: DBPool) -> impl Filter<Extract=(DBPool, ), Error=Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

pub enum Errors {
    MerchantError(String),
    AccountError(String),
    CustomerError(String),
    CardError(String),
    TransactionError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let log = warp::log("myLog");

    let pool = create_pool().unwrap();

    let token_route = warp::path!("api"/"token").and(warp::post())
        .and(with_db(pool.clone())).and(warp::body::json())
        .and_then(token::create_token_handler);

    let fund_route = warp::path!("api"/"account"/"fund").and(warp::post())
        .and(with_db(pool.clone())).and(warp::header("Authorization"))
        .and(warp::body::json()).and_then(transaction::fund_account_handler);

    let create_customer = warp::path!("api"/"customer").and(warp::post())
        .and(with_db(pool.clone())).and(warp::header("Authorization"))
        .and(warp::body::json()).and_then(customer::create_handler);

    let create_card = warp::path!("api"/"card").and(warp::post())
        .and(with_db(pool.clone())).and(warp::header("Authorization"))
        .and(warp::body::json()).and_then(card::create_virtual_handler);

    let deposit_card = warp::path!("api"/"card"/"deposit").and(warp::post())
        .and(with_db(pool.clone())).and(warp::header("Authorization"))
        .and(warp::body::json()).and_then(card::deposit_virtual_handler);

    let withdraw_card = warp::path!("api"/"card"/"withdraw").and(warp::post())
        .and(with_db(pool.clone())).and(warp::header("Authorization"))
        .and(warp::body::json()).and_then(card::withdraw_virtual_handler);

    let routes = token_route.or(fund_route).or(create_customer)
        .or(create_card).or(deposit_card).or(withdraw_card).with(log);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
