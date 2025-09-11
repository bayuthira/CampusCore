use crate::{
    db::DbPool,
    errors::AppError,
    modules::user_management::model::UserLookup,
};

pub async fn search_users_repo(pool: &DbPool, search_term: &str) -> Result<Vec<UserLookup>, AppError> {
    let search_pattern = format!("%{}%", search_term);

    let users = sqlx::query_as!(
        UserLookup,
        "SELECT id, username, full_name FROM users WHERE username ILIKE $1 OR full_name ILIKE $1 ORDER BY full_name LIMIT 10",
        search_pattern
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}