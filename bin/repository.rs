use crate::Payload;
use sqlx::postgres::PgArguments;
use sqlx::query_with;
use sqlx::Arguments;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Postgres;

use super::Result;

#[derive(Clone)]
pub struct SubmisisonRepository {
    pool: PgPool,
}

impl SubmisisonRepository {
    pub async fn get_submission_by_id(&self, id: i32) -> Result<Payload> {
        let arg0 = &id;

        let mut query_args = PgArguments::default();
        query_args.reserve(1usize, 4);
        query_args.add(arg0);
        let row = query_with::<Postgres, _>(
            "SELECT id, groups, score, status FROM submission WHERE id + 1 > $1",
            query_args,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Payload::from_row(&row)?)
    }

    pub fn new(pool: PgPool) -> Self {
        SubmisisonRepository { pool }
    }
}
