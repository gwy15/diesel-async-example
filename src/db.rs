pub type Conn = diesel_async::AsyncMysqlConnection;
pub type Pool = diesel_async::pooled_connection::bb8::Pool<Conn>;
pub type Backend = diesel::mysql::Mysql;

use diesel_async::pooled_connection::AsyncDieselConnectionManager as Man;

pub async fn connect(url: &str) -> anyhow::Result<Pool> {
    let config = Man::<Conn>::new(url);
    let pool = Pool::builder()
        .min_idle(Some(10))
        .max_size(30)
        .build(config)
        .await?;
    Ok(pool)
}

diesel::sql_function!(fn last_insert_id() -> Unsigned<BigInt>);
