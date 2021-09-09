use std::str::FromStr;
use mobc_postgres::{
    tokio_postgres::Error,
    tokio_postgres::{Config, NoTls},
    PgConnectionManager,
    mobc::Pool,
    mobc,
    mobc::Connection,
};

const CONN_URL: &str ="postgres://card_api:card_api@localhost:5432/card_api_rust";
const DB_POOL_MAX_OPEN: u64 = 20;

pub type DBPool = Pool<PgConnectionManager<NoTls>>;
pub type DBConn = Connection<PgConnectionManager<NoTls>>;

pub fn create_pool() -> Result<DBPool, mobc::Error<Error>> {
    let config = Config::from_str(CONN_URL).unwrap();
    let manager = PgConnectionManager::new(config, NoTls);
    Ok(Pool::builder().max_open(DB_POOL_MAX_OPEN).build(manager))
}

pub async fn get_db_conn(pool: &DBPool) -> DBConn {
    pool.get().await.unwrap()
}