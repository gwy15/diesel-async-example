#[macro_use]
extern crate tracing;

pub mod db;
pub mod models;
pub mod schema;

pub(crate) mod prelude {
    pub use crate::{db, models, schema};
    pub use anyhow::{anyhow, bail, Context, Result};
    pub use chrono::{DateTime, Utc};
    pub use db::{Conn, Pool};
    pub use diesel::associations::HasTable;
    pub use diesel::{ExpressionMethods, Insertable, OptionalExtension, QueryDsl, Queryable};
    pub use diesel_async::RunQueryDsl;

    pub trait Executor: diesel_async::AsyncConnection<Backend = db::Backend> {}
    impl<T> Executor for T where T: diesel_async::AsyncConnection<Backend = db::Backend> {}
}
use std::sync::Arc;

use prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    tracing_subscriber::fmt::init();

    let pool = db::connect(&dotenv::var("DATABASE_URL")?).await?;
    let pool = Arc::new(pool);

    let mut handles = vec![];
    for th in 0..100 {
        let pool = pool.clone();
        let handle = tokio::spawn(insert_and_validate(th, 1_000, pool));
        handles.push(handle);
    }
    futures::future::try_join_all(handles).await?;

    Ok(())
}

async fn insert_and_validate(thread_id: usize, count: usize, pool: Arc<Pool>) -> Result<()> {
    for i in 0..count {
        let name = format!("name-{thread_id}-{i}");
        let email = format!("email-{thread_id}-{i}");

        let id = models::User::insert(
            models::NewUser {
                name: &name,
                email: &email,
            },
            &mut pool.get().await?,
        )
        .await?;
        let user = models::User::from_id(id, &mut pool.get().await?)
            .await?
            .context("no user")?;
        assert_eq!(user.id, id);
        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
        debug!(thread_id, i, "pass");

        let rows_affected = models::User::delete(id, &mut pool.get().await?).await?;
        assert_eq!(rows_affected, 1);
    }

    Ok(())
}
