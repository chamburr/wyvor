use crate::db::get_pg_conn;

use rocket::Rocket;

embed_migrations!();

pub fn run_migrations(rocket: &Rocket) {
    let conn = get_pg_conn(rocket);
    embedded_migrations::run(&*conn).expect("Failed to run database migrations");
}
