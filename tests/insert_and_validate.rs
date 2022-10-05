use diesel_async_example::prelude::*;
use std::sync::Arc;
use tracing::*;

/// how many threads / futures are running at the same time
const THREAD_COUNT: usize = 100;
/// for each thread / future, how many iterations to go
const ITERATIONS: usize = 200;

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    tracing_subscriber::fmt::try_init().ok();

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
    .await
    .unwrap(); // unwrap so that the error message shows

    #[cfg(feature = "sync")]
    let handles = (0..THREAD_COUNT)
        .map(|th| {
            let pool = pool.clone();
            std::thread::spawn(move || insert_and_validate(th, ITERATIONS, pool))
        })
        .collect::<Vec<_>>();
    #[cfg(feature = "sync")]
    for handle in handles {
        // unwrap so that the error message shows
        handle.join().unwrap().unwrap();
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

        debug!("thread {} done {}/{}", thread_id, i, count);

        let id = models::User::cyclic_create(&mut conn).await?;
        let user = models::User::from_id(id, &mut conn)
            .await?
            .context("no user")?;
        assert_eq!(id, user.id);
        assert_eq!(user.name, format!("cyclic-{}", user.id));
        assert_eq!(user.email, format!("cyclic-{}", user.id));
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
