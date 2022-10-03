pub mod db;
pub mod models;
pub mod schema;

pub mod prelude {
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
