use sqlx::Row;

pub async fn migrate_token_table(pool: &sqlx::SqlitePool) -> Result<(), String> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tokens (
            name TEXT PRIMARY KEY,
            token TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

    Ok(())
}

pub async fn get_moetran_token(pool: &sqlx::SqlitePool) -> Result<String, String> {
    let row = sqlx::query("SELECT moetoken FROM tokens WHERE name = 'moetran_token'")
        .fetch_optional(pool)
        .await
        .map_err(|err| format!("Failed to get MoeToken from database: {}", err))?;

    match row {
        Some(row) => {
            let token: String = row.try_get("token").map_err(|err| {
                format!("Failed to read 'moetran_token' from database row: {}", err)
            })?;

            Ok(token)
        }
        None => Err("No 'moetran_token' found in database".to_string()),
    }
}

pub async fn save_moetran_token(pool: &sqlx::SqlitePool, token: &str) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO tokens (name, token, updated_at)
        VALUES ('moetran_token', ?, strftime('%s', 'now'))
        ON CONFLICT(id) DO UPDATE SET
            token = excluded.token,
            updated_at = excluded.updated_at;
        "#,
    )
    .bind(token)
    .execute(pool)
    .await
    .map_err(|err| format!("Failed to save MoeToken to database: {}", err))?;

    Ok(())
}

pub async fn remove_moetran_token(pool: &sqlx::SqlitePool) -> Result<(), String> {
    sqlx::query("DELETE FROM tokens WHERE name = 'moetran_token'")
        .execute(pool)
        .await
        .map_err(|err| format!("Failed to remove MoeToken from database: {}", err))?;

    Ok(())
}

pub async fn get_poprako_token(pool: &sqlx::SqlitePool) -> Result<String, String> {
    let row = sqlx::query("SELECT token FROM tokens WHERE name = 'poprako_token'")
        .fetch_optional(pool)
        .await
        .map_err(|err| format!("Failed to get Poprako token from database: {}", err))?;

    match row {
        Some(row) => {
            let token: String = row.try_get("token").map_err(|err| {
                format!("Failed to read 'poprako_token' from database row: {}", err)
            })?;

            Ok(token)
        }
        None => Err("No 'poprako_token' found in database".to_string()),
    }
}

pub async fn save_poprako_token(pool: &sqlx::SqlitePool, token: &str) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO tokens (name, token, updated_at)
        VALUES ('poprako_token', ?, strftime('%s', 'now'))
        ON CONFLICT(id) DO UPDATE SET
            token = excluded.token,
            updated_at = excluded.updated_at;
        "#,
    )
    .bind(token)
    .execute(pool)
    .await
    .map_err(|err| format!("Failed to save Poprako token to database: {}", err))?;

    Ok(())
}

pub async fn remove_poprako_token(pool: &sqlx::SqlitePool) -> Result<(), String> {
    sqlx::query("DELETE FROM tokens WHERE name = 'poprako_token'")
        .execute(pool)
        .await
        .map_err(|err| format!("Failed to remove Poprako token from database: {}", err))?;

    Ok(())
}
