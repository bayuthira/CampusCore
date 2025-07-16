// src/repositories/user_management_repo.rs
use super::{
    model::{RoleAssignmentPayload, UserWithRoles, UpdateUserPayload,CreateUserPayload},
};

use crate::{
    db::DbPool,
    errors::AppError,
};
use futures::future::try_join_all;
use uuid::Uuid;

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

pub async fn get_user_by_id_with_roles_repo(pool: &DbPool, id: Uuid) -> Result<UserWithRoles, AppError> {
    let user = sqlx::query!("SELECT id, full_name, username, email, is_active, created_at FROM users WHERE id = $1", id)
        .fetch_one(pool)
        .await?;

    let roles = sqlx::query!(
        "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1",
        user.id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|rec| rec.name)
    .collect();

    Ok(UserWithRoles {
        id: user.id,
        full_name: user.full_name,
        username: user.username,
        email: user.email,
        is_active: user.is_active,
        created_at: user.created_at,
        roles,
    })
}

pub async fn update_user_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateUserPayload,
) -> Result<UserWithRoles, AppError> {
    // Mulai transaksi
    let mut tx = pool.begin().await?;

    // 1. Update data dasar di tabel `users`
    sqlx::query!(
        "UPDATE users SET full_name = $1, email = $2, is_active = $3, updated_at = now() WHERE id = $4",
        payload.full_name,
        payload.email,
        payload.is_active,
        id
    )
    .execute(&mut *tx)
    .await?;

    // 2. Lakukan sinkronisasi peran: Hapus semua peran lama user ini
    sqlx::query!("DELETE FROM user_roles WHERE user_id = $1", id)
        .execute(&mut *tx)
        .await?;

    // 3. Masukkan semua peran baru dari payload
    if !payload.role_ids.is_empty() {
        // `unnest` adalah cara efisien di PostgreSQL untuk memasukkan banyak baris sekaligus
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id) SELECT $1, unnest($2::uuid[])",
            id,
            &payload.role_ids
        )
        .execute(&mut *tx)
        .await?;
    }

    // 4. Commit transaksi jika semua berhasil
    tx.commit().await?;

    // Ambil dan kembalikan data user terbaru yang sudah lengkap dengan peran barunya
    let updated_user = get_user_by_id_with_roles_repo(pool, id).await?;
    Ok(updated_user)
}

pub async fn delete_user_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    // Berkat `ON DELETE CASCADE` di tabel `user_roles`,
    // semua relasi peran user ini akan ikut terhapus secara otomatis.
    let rows_affected = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    Ok(())
}


pub async fn reset_user_password_repo(
    pool: &DbPool,
    id: Uuid,
    hashed_password: &str,
) -> Result<(), AppError> {
    let rows_affected = sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = now() WHERE id = $2",
        hashed_password,
        id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    Ok(())
}

pub async fn create_user_repo(pool: &DbPool, payload: CreateUserPayload) -> Result<UserWithRoles, AppError> {
    // Mulai transaksi
    let mut tx = pool.begin().await?;

    // 1. Hash password
    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST)?;

    // 2. Insert ke tabel `users`
    let new_user_id = sqlx::query_scalar!(
        "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
        payload.username,
        hashed_password,
        payload.full_name,
        payload.email
    )
    .fetch_one(&mut *tx)
    .await?;

    // 3. Loop dan insert ke tabel `user_roles`
    for role_id in payload.role_ids {
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)",
            new_user_id,
            role_id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Commit transaksi jika semua berhasil
    tx.commit().await?;

    // Ambil dan kembalikan detail user yang baru dibuat
    let new_user = get_user_by_id_with_roles_repo(pool, new_user_id).await?;
    Ok(new_user)
}