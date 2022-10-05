use crate::prelude::*;
use schema::user::dsl;

#[derive(Insertable)]
#[diesel(table_name = schema::user)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = schema::user)]
pub struct UserChange<'a> {
    pub id: u64,
    pub name: Option<&'a str>,
    pub email: Option<&'a str>,
}

impl User {
    #[cfg(feature = "async")]
    pub async fn insert<C: Executor>(new: NewUser<'_>, conn: &mut C) -> Result<u64> {
        diesel::insert_into(schema::user::dsl::user)
            .values(&new)
            .execute(conn)
            .await?;
        let id = diesel::select(db::last_insert_id())
            .get_result::<u64>(conn)
            .await?;
        Ok(id)
    }

    #[cfg(feature = "async")]
    pub async fn from_id<C: Executor>(id: u64, conn: &mut C) -> Result<Option<Self>> {
        let user = dsl::user
            .filter(dsl::id.eq(id))
            .get_result::<Self>(conn)
            .await
            .optional()?;

        Ok(user)
    }

    #[cfg(feature = "async")]
    pub async fn delete<C: Executor>(id: u64, conn: &mut C) -> Result<usize> {
        let rows_affected = diesel::delete(dsl::user.filter(dsl::id.eq(id)))
            .execute(conn)
            .await?;
        Ok(rows_affected)
    }

    #[cfg(feature = "async")]
    pub async fn update<C: Executor>(update: UserChange<'_>, conn: &mut C) -> Result<()> {
        diesel::update(dsl::user.filter(dsl::id.eq(update.id)))
            .set(&update)
            .execute(conn)
            .await?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn cyclic_create<C: Executor>(conn: &mut C) -> Result<u64> {
        let id = conn
            .transaction::<_, Error, _>(|conn| {
                async move {
                    let new = NewUser {
                        name: "tmp",
                        email: "tmp",
                    };
                    let id = Self::insert(new, conn).await?;
                    let new_name = format!("cyclic-{}", id);
                    let update = UserChange {
                        id,
                        name: Some(&new_name),
                        email: Some(&new_name),
                    };
                    Self::update(update, conn).await?;
                    Ok(id)
                }
                .boxed()
            })
            .await?;

        Ok(id)
    }

    #[cfg(feature = "sync")]
    pub fn insert<C: Executor>(new: NewUser<'_>, conn: &mut C) -> Result<u64> {
        diesel::insert_into(schema::user::dsl::user)
            .values(&new)
            .execute(conn)?;
        let id = diesel::select(db::last_insert_id()).get_result::<u64>(conn)?;
        Ok(id)
    }

    #[cfg(feature = "sync")]
    pub fn from_id<C: Executor>(id: u64, conn: &mut C) -> Result<Option<Self>> {
        let user = dsl::user
            .filter(dsl::id.eq(id))
            .get_result::<Self>(conn)
            .optional()?;

        Ok(user)
    }

    #[cfg(feature = "sync")]
    pub fn delete<C: Executor>(id: u64, conn: &mut C) -> Result<usize> {
        let rows_affected = diesel::delete(dsl::user.filter(dsl::id.eq(id))).execute(conn)?;
        Ok(rows_affected)
    }

    #[cfg(feature = "sync")]
    pub fn update<C: Executor>(update: UserChange<'_>, conn: &mut C) -> Result<()> {
        diesel::update(dsl::user.filter(dsl::id.eq(update.id)))
            .set(&update)
            .execute(conn)?;
        Ok(())
    }
}
