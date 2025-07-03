// src/repositories/user_management_repo.rs
use crate::{
    db::DbPool,
    errors::AppError,
    models::user_model::{RoleAssignmentPayload, UserWithRoles},
};
use futures::future::try_join_all;

// Fungsi untuk mendapatkan semua user beserta perannya
pub async fn get_all_users_with_roles_repo(pool: &DbPool) -> Result<Vec<UserWithRoles>, AppError> {
    let users = sqlx::query!("SELECT id, full_name, username, email, is_active, created_at FROM users ORDER BY full_name")
        .fetch_all(pool)
        .await?;

    let users_with_roles_futures: Vec<_> = users
        .into_iter()
        .map(|user| async move { // `move` adalah praktik yang baik di sini
            let roles = sqlx::query!(
                "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1",
                user.id
            )
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|rec| rec.name)
            .collect();

            // --- PERBAIKAN DI SINI ---
            // Beri tahu kompiler secara eksplisit tipe dari `Result` ini
            let result: Result<UserWithRoles, AppError> = Ok(UserWithRoles {
                id: user.id,
                full_name: user.full_name,
                username: user.username,
                email: user.email,
                is_active: user.is_active,
                created_at: user.created_at,
                roles,
            });

            result // Kembalikan hasil yang sudah jelas tipenya
        })
        .collect();
    
    let users_with_roles = try_join_all(users_with_roles_futures).await?;
    Ok(users_with_roles)
}


// Fungsi untuk memberikan peran ke user
pub async fn assign_role_to_user_repo(pool: &DbPool, payload: RoleAssignmentPayload) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        payload.user_id,
        payload.role_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Fungsi untuk mencabut peran dari user
pub async fn revoke_role_from_user_repo(pool: &DbPool, payload: RoleAssignmentPayload) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2",
        payload.user_id,
        payload.role_id
    )
    .execute(pool)
    .await?;
    Ok(())
}