use std::time::Duration;

use indoc::{formatdoc, indoc};
use sqlx::{Sqlite, migrate::MigrateDatabase};

use crate::{
    agent::ChatMessage,
    indoc_error, indoc_info, indoc_warn,
    states::{DATA_DIR, DB_POOL, SERVER_CONFIG},
};

pub async fn init_sqlite_pool(max_conn: u32) -> anyhow::Result<sqlx::Pool<Sqlite>> {
    let data_dir = DATA_DIR.get().unwrap();
    let db_path = data_dir.join("store.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    if Sqlite::database_exists(&db_url).await? {
        indoc_info!("Building DB connection pool from existing database...");
    } else {
        Sqlite::create_database(&db_url).await?;
        indoc_info!("Building DB connection pool from newly created database...");
    }
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(max_conn)
        .connect(&db_url)
        .await?;
    indoc_info!("DB init complete.");
    Ok(pool)
}

pub async fn init_chat_history_table() {
    let pool = DB_POOL.get().unwrap();
    let query = indoc!(
        "
        CREATE TABLE IF NOT EXISTS chat_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL,
            message TEXT,
            role TEXT NOT NULL,
            time DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_user_created_at ON chat_history (uuid, time);
        "
    );
    if let Err(e) = sqlx::query(query).execute(pool).await {
        indoc_error!(
            "
            Init chat history table failed, error:
            {e}
            "
        );
    }
}

pub async fn clear_history_by_uuid(uuid: &str) {
    let pool = DB_POOL.get().unwrap();
    let query = indoc!(
        "
        DELETE FROM chat_history
        WHERE uuid = $1;
        "
    );
    if let Err(e) = sqlx::query(query).bind(uuid).execute(pool).await {
        indoc_warn!(
            "
            Clear chat history by uuid failed, error:
            {e}
            "
        );
    }
}

pub async fn block_periodic_clear_history() {
    let mut interval = tokio::time::interval(Duration::from_secs(600));
    loop {
        interval.tick().await;
        let rows = clear_old_history().await;
        indoc_info!("Scheduled clear history: {rows} rows removed.");
    }
}

pub async fn clear_old_history() -> u64 {
    let pool = DB_POOL.get().unwrap();
    let config = SERVER_CONFIG.get().unwrap();
    let query = formatdoc!(
        "
        DELETE FROM chat_history
        WHERE uuid IN (
            SELECT uuid FROM (
                SELECT uuid
                FROM chat_history
                GROUP BY uuid
                HAVING MAX(time) < datetime('now', '-{} days')
            )
        );
        ",
        config.chat_expire_days
    );
    match sqlx::query(&query).execute(pool).await {
        Ok(r) => {
            r.rows_affected()
        }
        Err(e) => {
            indoc_warn!(
                "
                Clear old history failed, error:
                {e}
                "
            );
            0
        }
    }
}

impl ChatMessage {
    pub async fn load_all(uuid: &str) -> Vec<Self> {
        let pool = DB_POOL.get().unwrap();
        let query = indoc!(
            "
            SELECT 
                uuid, 
                message, 
                role
            FROM chat_history
            WHERE uuid = $1
            ORDER BY time ASC;
            "
        );
        match sqlx::query_as(query).bind(uuid).fetch_all(pool).await {
            Ok(list) => list,
            Err(e) => {
                indoc_warn!(
                    "
                    query chat history failed, error:
                    {e}
                    "
                );
                Vec::new()
            }
        }
    }

    pub async fn persist(&self) {
        let pool = DB_POOL.get().unwrap();
        let query = indoc!(
            "
            INSERT INTO chat_history (uuid, message, role)
            VALUES ($1, $2, $3);
            "
        );
        if let Err(e) = sqlx::query(query)
            .bind(self.uuid.clone())
            .bind(self.content.clone())
            .bind(self.role.clone())
            .execute(pool)
            .await
        {
            indoc_warn!(
                "
                Insert chat history failed, error:
                {e}
                "
            );
        };
    }
}
