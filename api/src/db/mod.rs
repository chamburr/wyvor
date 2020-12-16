use rocket::Rocket;
use rocket_contrib::databases::{diesel, redis};

pub mod cache;
pub mod migration;
pub mod pubsub;
pub mod schema;

#[database("postgres")]
pub struct PgConn(diesel::PgConnection);

#[database("redis")]
pub struct RedisConn(redis::Connection);

pub fn get_pg_conn(rocket: &Rocket) -> PgConn {
    PgConn::get_one(rocket).expect("Failed to get postgres connection")
}

pub fn get_redis_conn(rocket: &Rocket) -> RedisConn {
    RedisConn::get_one(rocket).expect("Failed to get redis connection")
}
