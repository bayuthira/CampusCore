use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use uuid::Uuid;

#[derive(Debug)]
pub struct AsesmenAccess {
    pub assigned: bool,
    pub coordinator: bool,
    pub kaprodi: bool,
    pub academic: bool,
    pub production: bool,
}

pub fn has_role(claims: &TokenClaims, role: &str) -> bool {
    claims.roles.iter().any(|item| item == role)
}

pub async fn kaprodi_prodi_ids(pool: &DbPool, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    let rows = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT COALESCE(NULLIF(ur.context->>'prodi_id', '')::UUID, d.prodi_id)
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id AND r.name = 'KAPRODI'
        LEFT JOIN pegawai p ON p.user_id = ur.user_id
        LEFT JOIN dosen d ON d.pegawai_id = p.id
        WHERE ur.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().flatten().collect())
}

pub async fn dosen_id(pool: &DbPool, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT d.id FROM dosen d JOIN pegawai p ON p.id = d.pegawai_id WHERE p.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn for_jadwal(
    pool: &DbPool,
    claims: &TokenClaims,
    jadwal_id: Uuid,
) -> Result<AsesmenAccess, AppError> {
    let (assigned, coordinator, prodi_id) = sqlx::query_as::<_, (bool, bool, Uuid)>(
        r#"
        SELECT
            EXISTS(
                SELECT 1 FROM jadwal_dosen_pengampu jdp
                JOIN dosen d ON d.id = jdp.dosen_id
                JOIN pegawai p ON p.id = d.pegawai_id
                WHERE jdp.jadwal_kuliah_id = jk.id AND p.user_id = $2
            ),
            EXISTS(
                SELECT 1 FROM jadwal_dosen_pengampu jdp
                JOIN dosen d ON d.id = jdp.dosen_id
                JOIN pegawai p ON p.id = d.pegawai_id
                WHERE jdp.jadwal_kuliah_id = jk.id AND p.user_id = $2
                  AND jdp.peran::TEXT = 'Koordinator'
            ),
            mk.prodi_id
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        WHERE jk.id = $1
        "#,
    )
    .bind(jadwal_id)
    .bind(claims.sub)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

    let academic = has_role(claims, "SUPER_ADMIN") || has_role(claims, "STAF_AKADEMIK");
    let production = has_role(claims, "SUPER_ADMIN") || has_role(claims, "STAF_BAUM");
    let kaprodi = if has_role(claims, "SUPER_ADMIN") {
        true
    } else if has_role(claims, "KAPRODI") {
        kaprodi_prodi_ids(pool, claims.sub)
            .await?
            .contains(&prodi_id)
    } else {
        false
    };
    Ok(AsesmenAccess {
        assigned,
        coordinator: coordinator || has_role(claims, "SUPER_ADMIN"),
        kaprodi,
        academic,
        production,
    })
}

pub async fn jadwal_for_asesmen(pool: &DbPool, asesmen_id: Uuid) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>("SELECT jadwal_kuliah_id FROM asesmen_kuliah WHERE id = $1")
        .bind(asesmen_id)
        .fetch_optional(pool)
        .await?
        .ok_or(sqlx::Error::RowNotFound.into())
}

pub async fn for_asesmen(
    pool: &DbPool,
    claims: &TokenClaims,
    asesmen_id: Uuid,
) -> Result<AsesmenAccess, AppError> {
    let jadwal_id = jadwal_for_asesmen(pool, asesmen_id).await?;
    for_jadwal(pool, claims, jadwal_id).await
}

fn forbidden(message: &str) -> Result<(), AppError> {
    Err(AppError::Forbidden(message.to_string()))
}

pub fn require_view(access: &AsesmenAccess) -> Result<(), AppError> {
    if access.assigned || access.kaprodi || access.academic || access.production {
        Ok(())
    } else {
        forbidden("Anda tidak memiliki akses ke asesmen ini.")
    }
}

pub fn require_edit(access: &AsesmenAccess) -> Result<(), AppError> {
    if access.coordinator || access.academic {
        Ok(())
    } else {
        forbidden("Hanya koordinator atau staf akademik yang dapat mengubah jadwal asesmen.")
    }
}

pub fn require_content(access: &AsesmenAccess) -> Result<(), AppError> {
    if access.coordinator {
        Ok(())
    } else {
        forbidden("Hanya koordinator mata kuliah yang dapat mengelola naskah asesmen.")
    }
}

pub fn require_review(access: &AsesmenAccess) -> Result<(), AppError> {
    if access.kaprodi {
        Ok(())
    } else {
        forbidden("Hanya Kaprodi terkait yang dapat meninjau asesmen.")
    }
}

pub fn require_production(access: &AsesmenAccess) -> Result<(), AppError> {
    if access.production {
        Ok(())
    } else {
        forbidden("Anda tidak memiliki akses penggandaan soal.")
    }
}

pub fn require_execute(access: &AsesmenAccess) -> Result<(), AppError> {
    if access.assigned || access.academic {
        Ok(())
    } else {
        forbidden("Hanya dosen pengampu atau staf akademik yang dapat melaksanakan ujian.")
    }
}

pub fn require_grade(access: &AsesmenAccess) -> Result<(), AppError> {
    if access.coordinator {
        Ok(())
    } else {
        forbidden("Hanya koordinator mata kuliah yang dapat memasukkan nilai.")
    }
}
