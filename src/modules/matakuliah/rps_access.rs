use super::rps_model::RpsMataKuliahAccess;
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use uuid::Uuid;

fn has_role(claims: &TokenClaims, role: &str) -> bool {
    claims.roles.iter().any(|item| item == role)
}

async fn kaprodi_prodi_ids(pool: &DbPool, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
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

pub async fn list_for_user(
    pool: &DbPool,
    claims: &TokenClaims,
) -> Result<Vec<RpsMataKuliahAccess>, AppError> {
    let is_super_admin = has_role(claims, "SUPER_ADMIN");
    let is_kaprodi = has_role(claims, "KAPRODI");
    let prodi_ids = if is_kaprodi {
        kaprodi_prodi_ids(pool, claims.sub).await?
    } else {
        Vec::new()
    };

    Ok(sqlx::query_as::<_, RpsMataKuliahAccess>(
        r#"
        SELECT mk.id, mk.kode_mk, mk.nama_mk, mk.prodi_id, p.nama_prodi,
               mk.file_rps_path, mk.status_verifikasi_rps, mk.catatan_verifikasi_rps,
               CASE
                   WHEN COALESCE(BOOL_OR(jdp.peran::TEXT = 'Koordinator'), false) THEN 'Koordinator'
                   WHEN COALESCE(BOOL_OR(jdp.peran::TEXT = 'Anggota'), false) THEN 'Anggota'
                   WHEN $2 THEN 'Kaprodi'
                   WHEN $1 THEN 'Administrator'
                   ELSE NULL
               END AS peran_pengampu,
               ($1 OR COALESCE(BOOL_OR(jdp.peran::TEXT = 'Koordinator'), false)) AS can_edit,
               ($1 OR ($2 AND mk.prodi_id = ANY($3))) AS can_verify
        FROM mata_kuliah mk
        JOIN prodi p ON p.id = mk.prodi_id
        LEFT JOIN jadwal_kuliah jk ON jk.matakuliah_id = mk.id
        LEFT JOIN tahun_akademik ta
            ON ta.id = jk.tahun_akademik_id AND ta.is_active = true
        LEFT JOIN jadwal_dosen_pengampu jdp
            ON jdp.jadwal_kuliah_id = jk.id
           AND ta.id IS NOT NULL
           AND jdp.dosen_id = (
               SELECT d.id FROM dosen d
               JOIN pegawai pg ON pg.id = d.pegawai_id
               WHERE pg.user_id = $4 LIMIT 1
           )
        WHERE $1 OR ($2 AND mk.prodi_id = ANY($3)) OR jdp.dosen_id IS NOT NULL
        GROUP BY mk.id, p.nama_prodi
        ORDER BY mk.kode_mk
        "#,
    )
    .bind(is_super_admin)
    .bind(is_kaprodi)
    .bind(&prodi_ids)
    .bind(claims.sub)
    .fetch_all(pool)
    .await?)
}

async fn access_flags(
    pool: &DbPool,
    claims: &TokenClaims,
    mata_kuliah_id: Uuid,
) -> Result<(bool, bool, bool), AppError> {
    if has_role(claims, "SUPER_ADMIN") {
        return Ok((true, true, true));
    }

    let (is_assigned, is_coordinator, prodi_id) = sqlx::query_as::<_, (bool, bool, Uuid)>(
        r#"
            SELECT
                EXISTS(
                    SELECT 1 FROM jadwal_kuliah jk
                    JOIN tahun_akademik ta ON ta.id = jk.tahun_akademik_id
                    JOIN jadwal_dosen_pengampu jdp ON jdp.jadwal_kuliah_id = jk.id
                    JOIN dosen d ON d.id = jdp.dosen_id
                    JOIN pegawai p ON p.id = d.pegawai_id
                    WHERE jk.matakuliah_id = mk.id AND ta.is_active = true
                      AND p.user_id = $2
                ),
                EXISTS(
                    SELECT 1 FROM jadwal_kuliah jk
                    JOIN tahun_akademik ta ON ta.id = jk.tahun_akademik_id
                    JOIN jadwal_dosen_pengampu jdp ON jdp.jadwal_kuliah_id = jk.id
                    JOIN dosen d ON d.id = jdp.dosen_id
                    JOIN pegawai p ON p.id = d.pegawai_id
                    WHERE jk.matakuliah_id = mk.id AND ta.is_active = true
                      AND p.user_id = $2 AND jdp.peran::TEXT = 'Koordinator'
                ),
                mk.prodi_id
            FROM mata_kuliah mk WHERE mk.id = $1
            "#,
    )
    .bind(mata_kuliah_id)
    .bind(claims.sub)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

    let is_kaprodi_scope = if has_role(claims, "KAPRODI") {
        kaprodi_prodi_ids(pool, claims.sub)
            .await?
            .contains(&prodi_id)
    } else {
        false
    };
    Ok((
        is_assigned || is_kaprodi_scope,
        is_coordinator,
        is_kaprodi_scope,
    ))
}

pub async fn assert_can_view(
    pool: &DbPool,
    claims: &TokenClaims,
    mata_kuliah_id: Uuid,
) -> Result<(), AppError> {
    if access_flags(pool, claims, mata_kuliah_id).await?.0 {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Anda tidak memiliki akses ke RPS mata kuliah ini.".to_string(),
        ))
    }
}

pub async fn assert_can_edit(
    pool: &DbPool,
    claims: &TokenClaims,
    mata_kuliah_id: Uuid,
) -> Result<(), AppError> {
    if has_role(claims, "SUPER_ADMIN") || access_flags(pool, claims, mata_kuliah_id).await?.1 {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Hanya koordinator mata kuliah yang dapat mengubah RPS.".to_string(),
        ))
    }
}

pub async fn assert_can_verify(
    pool: &DbPool,
    claims: &TokenClaims,
    mata_kuliah_id: Uuid,
) -> Result<(), AppError> {
    if has_role(claims, "SUPER_ADMIN") || access_flags(pool, claims, mata_kuliah_id).await?.2 {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "RPS hanya dapat diverifikasi oleh Kaprodi terkait.".to_string(),
        ))
    }
}

pub async fn mata_kuliah_id_for_weekly(pool: &DbPool, weekly_id: Uuid) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT mata_kuliah_id FROM mata_kuliah_rps_mingguan WHERE id = $1",
    )
    .bind(weekly_id)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound.into())
}
