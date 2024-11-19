use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

#[derive(Debug)]
pub struct Database {
    pub db_connection: Pool<Postgres>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_uri = env::var("DB_URI").expect("failed to read DB_URI env variable");
        let connection = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_uri)
            .await?;

        println!("connected to database");

        Ok(Self {
            db_connection: connection,
        })
    }

    pub async fn check_unit_is_valid(self: &Self, unit_id: String) -> Result<bool, sqlx::Error> {
        let db_query = format!("SELECT 1 FROM biometric WHERE (unit_id=$1)");

        let unit_id = unit_id.to_uppercase();

        let result = sqlx::query(&db_query)
            .bind(unit_id)
            .execute(&self.db_connection)
            .await?;

        if result.rows_affected() == 1 {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    pub async fn update_unit_state(
        self: &Self,
        unit_id: String,
        unit_state: bool,
    ) -> Result<bool, sqlx::Error> {
        let db_query = format!("UPDATE biometric SET online=$1 WHERE unit_id=$2");

        let unit_id = unit_id.to_uppercase();

        let result = sqlx::query(&db_query)
            .bind(unit_state)
            .bind(unit_id)
            .execute(&self.db_connection)
            .await?;
        if result.rows_affected() == 1 {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
}
