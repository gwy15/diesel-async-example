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

impl User {
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

    pub async fn from_id<C: Executor>(id: u64, conn: &mut C) -> Result<Option<Self>> {
        let user = dsl::user
            .filter(dsl::id.eq(id))
            .get_result::<Self>(conn)
            .await
            .optional()?;

        Ok(user)
    }

    pub async fn delete<C: Executor>(id: u64, conn: &mut C) -> Result<usize> {
        let rows_affected = diesel::delete(dsl::user.filter(dsl::id.eq(id)))
            .execute(conn)
            .await?;
        Ok(rows_affected)
    }
}
