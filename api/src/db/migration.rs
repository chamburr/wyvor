use crate::db::PgPool;
use crate::routes::ApiResult;

embed_migrations!();

pub async fn run_migrations(pool: &PgPool) -> ApiResult<()> {
    let conn = pool.get()?;
    embedded_migrations::run(&*conn)?;

    Ok(())
}
