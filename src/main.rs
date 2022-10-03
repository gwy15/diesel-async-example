#[macro_use]
extern crate tracing;

pub mod db;
pub mod models;
pub mod schema;

pub(crate) mod prelude {
    pub use crate::{db, models, schema};
    pub use anyhow::{anyhow, bail, Context, Error, Result};
    pub use chrono::{DateTime, Utc};
    pub use db::{Conn, Pool};
    pub use diesel::associations::HasTable;
    pub use diesel::{
        AsChangeset, ExpressionMethods, Identifiable, Insertable, OptionalExtension, QueryDsl,
        Queryable,
    };
    pub use futures::FutureExt;

    #[cfg(feature = "async")]
    pub use diesel_async::RunQueryDsl;
    #[cfg(feature = "async")]
    pub trait Executor: diesel_async::AsyncConnection<Backend = db::Backend> {}
    #[cfg(feature = "async")]
    impl<T> Executor for T where T: diesel_async::AsyncConnection<Backend = db::Backend> {}

    #[cfg(feature = "sync")]
    pub use diesel::RunQueryDsl;
    #[cfg(feature = "sync")]
    pub trait Executor:
        diesel::Connection<Backend = db::Backend> + diesel::connection::LoadConnection
    {
    }
    #[cfg(feature = "sync")]
    impl<T> Executor for T where
        T: diesel::Connection<Backend = db::Backend> + diesel::connection::LoadConnection
    {
    }
}
use std::sync::Arc;

use prelude::*;

/// how many threads / futures are running at the same time
const THREAD_COUNT: usize = 50;
/// for each thread / future, how many iterations to go
const ITERATIONS: usize = 1000;
/// max connections made to mysql
const MAX_CONNECTIONS: u32 = 30;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    tracing_subscriber::fmt::init();

    let url = dotenv::var("DATABASE_URL")?;
    #[cfg(feature = "async")]
    let pool = db::connect(&url).await?;
    #[cfg(feature = "sync")]
    let pool = db::connect(&url)?;
    let pool = Arc::new(pool);

    #[cfg(feature = "async")]
    futures::future::try_join_all(
        (0..THREAD_COUNT)
            .map(|th| {
                let pool = pool.clone();
                insert_and_validate(th, ITERATIONS, pool)
            })
            .collect::<Vec<_>>(),
    )
    .await?;

    #[cfg(feature = "sync")]
    let handles = (0..THREAD_COUNT)
        .map(|th| {
            let pool = pool.clone();
            std::thread::spawn(move || insert_and_validate(th, ITERATIONS, pool))
        })
        .collect::<Vec<_>>();
    #[cfg(feature = "sync")]
    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

#[cfg(feature = "async")]
async fn insert_and_validate(thread_id: usize, count: usize, pool: Arc<Pool>) -> Result<()> {
    for i in 0..count {
        let mut conn = pool.get().await?;

        let name = format!("name-{thread_id}-{i}");
        let email = format!("email-{thread_id}-{i}");

        let new = models::NewUser {
            name: &name,
            email: &email,
        };
        let id = models::User::insert(new, &mut conn).await?;
        let user = models::User::from_id(id, &mut conn)
            .await?
            .context("no user")?;

        assert_eq!(user.id, id);
        assert_eq!(user.name, name);
        assert_eq!(user.email, email);

        let rows_affected = models::User::delete(id, &mut conn).await?;
        assert_eq!(rows_affected, 1);

        // let user = models::User::cyclic_create(&mut conn).await?;
        // assert_eq!(user.name, format!("cyclic-{}", user.id));
    }
    debug!(thread_id, "pass");

    Ok(())
}

#[cfg(feature = "sync")]
fn insert_and_validate(thread_id: usize, count: usize, pool: Arc<Pool>) -> Result<()> {
    for i in 0..count {
        #[cfg(feature = "sync")]
        let mut conn = pool.get()?;

        let name = format!("name-{thread_id}-{i}");
        let email = format!("email-{thread_id}-{i}");

        let new = models::NewUser {
            name: &name,
            email: &email,
        };

        let id = models::User::insert(new, &mut conn)?;
        let user = models::User::from_id(id, &mut conn)?.context("no user")?;
        assert_eq!(user.id, id);
        assert_eq!(user.name, name);
        assert_eq!(user.email, email);

        let rows_affected = models::User::delete(id, &mut conn)?;
        assert_eq!(rows_affected, 1);

        // let user = models::User::cyclic_create(&mut conn).await?;
        // assert_eq!(user.name, format!("cyclic-{}", user.id));
    }
    debug!(thread_id, "pass");

    Ok(())
}
