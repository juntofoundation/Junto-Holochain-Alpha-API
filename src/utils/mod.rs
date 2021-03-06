use diesel::r2d2::{ConnectionManager, Pool};
use diesel::pg::PgConnection;
use std::env;

use crate::models;

pub mod holochain;
pub mod wrapper;

pub fn load_connection_pool() -> models::db::Pool{
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|e| {
        panic!("could not find {}: {}", "DATABASE_URL", e)
    });
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager).unwrap()
}