pub type Backend = diesel::mysql::Mysql;

#[cfg(feature = "async")]
pub type Conn = diesel_async::AsyncMysqlConnection;
#[cfg(feature = "async")]
pub type Pool = diesel_async::pooled_connection::bb8::Pool<Conn>;

#[cfg(feature = "sync")]
pub type Conn = diesel::MysqlConnection;
#[cfg(feature = "sync")]
pub type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<Conn>>;

#[cfg(feature = "async")]
pub async fn connect(url: &str) -> anyhow::Result<Pool> {
    let config = diesel_async::pooled_connection::AsyncDieselConnectionManager::<Conn>::new(url);
    let pool = Pool::builder()
        .max_size(super::MAX_CONNECTIONS)
        .build(config)
        .await?;
    Ok(pool)
}

#[cfg(feature = "sync")]
pub fn connect(url: &str) -> anyhow::Result<Pool> {
    let config = diesel::r2d2::ConnectionManager::<Conn>::new(url);
    let pool = Pool::builder()
        .max_size(super::MAX_CONNECTIONS)
        .build(config)?;
    Ok(pool)
}

diesel::sql_function!(fn last_insert_id() -> Unsigned<BigInt>);
