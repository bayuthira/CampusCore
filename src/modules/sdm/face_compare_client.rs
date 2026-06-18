use crate::errors::AppError;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use serde_json::Value;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceCompareProvider {
    FacePlusPlus,
    OpenCvSFace,
    Disabled,
}

#[derive(Debug)]
pub struct FaceVerifyResult {
    pub is_match: bool,
    pub similarity: f32,
    pub probe_embedding: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ExtractResponse {
    embedding: Value,
}

#[derive(Debug, Deserialize)]
struct VerifyFacePart {
    embedding: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    #[serde(rename = "match")]
    is_match: bool,
    similarity: f32,
    probe_face: Option<VerifyFacePart>,
}

pub fn provider_from_env() -> FaceCompareProvider {
    match env::var("FACE_COMPARE_PROVIDER")
        .unwrap_or_else(|_| "faceplusplus".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "opencv_sface" | "opencv" | "sface" | "face_compare_service" => {
            FaceCompareProvider::OpenCvSFace
        }
        "disabled" | "off" | "none" => FaceCompareProvider::Disabled,
        _ => FaceCompareProvider::FacePlusPlus,
    }
}

fn base_url() -> Result<String, AppError> {
    let configured = env::var("FACE_COMPARE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8088".to_string())
        .trim_end_matches('/')
        .to_string();

    let base = configured
        .strip_suffix("/verify-embedding")
        .or_else(|| configured.strip_suffix("/verify"))
        .or_else(|| configured.strip_suffix("/extract"))
        .unwrap_or(&configured)
        .trim_end_matches('/')
        .to_string();

    if base.is_empty() {
        return Err(AppError::InternalServerError(
            "FACE_COMPARE_URL belum dikonfigurasi.".to_string(),
        ));
    }

    Ok(base)
}

fn threshold_from_env() -> Option<String> {
    env::var("FACE_COMPARE_THRESHOLD")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn apply_auth(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    match env::var("FACE_COMPARE_AUTH_CODE") {
        Ok(auth_code) if !auth_code.trim().is_empty() => {
            builder.header("X-Auth-Code", auth_code.trim().to_string())
        }
        _ => builder,
    }
}

fn image_part(bytes: Vec<u8>, file_name: &'static str) -> Result<Part, AppError> {
    Part::bytes(bytes)
        .file_name(file_name)
        .mime_str("image/jpeg")
        .map_err(|e| AppError::AnyhowError(anyhow::anyhow!("Gagal membuat form foto: {}", e)))
}

async fn parse_face_service_error(response: reqwest::Response) -> AppError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    let message = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|json| {
            json.pointer("/detail/message")
                .and_then(Value::as_str)
                .or_else(|| json.get("message").and_then(Value::as_str))
                .or_else(|| json.get("error").and_then(Value::as_str))
                .map(str::to_string)
        })
        .unwrap_or_else(|| {
            if body.trim().is_empty() {
                format!("Face compare service mengembalikan status {}", status)
            } else {
                body
            }
        });

    AppError::Forbidden(format!("Face compare service menolak foto: {}", message))
}

pub async fn extract_embedding(image_bytes: Vec<u8>) -> Result<Value, AppError> {
    let client = reqwest::Client::new();
    let url = format!("{}/extract", base_url()?);
    let form = Form::new().part("image", image_part(image_bytes, "face.jpg")?);

    let response = apply_auth(client.post(url))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!(
                "Gagal menghubungi face compare service: {}",
                e
            ))
        })?;

    if !response.status().is_success() {
        return Err(parse_face_service_error(response).await);
    }

    let parsed: ExtractResponse = response.json().await.map_err(|e| {
        AppError::AnyhowError(anyhow::anyhow!(
            "Gagal parse respons /extract face compare service: {}",
            e
        ))
    })?;

    Ok(parsed.embedding)
}

pub async fn verify_embedding(
    reference_embedding: &Value,
    probe_bytes: Vec<u8>,
) -> Result<FaceVerifyResult, AppError> {
    let client = reqwest::Client::new();
    let url = format!("{}/verify-embedding", base_url()?);
    let mut form = Form::new()
        .text("reference_embedding", reference_embedding.to_string())
        .part("probe_image", image_part(probe_bytes, "absensi.jpg")?);

    if let Some(threshold) = threshold_from_env() {
        form = form.text("threshold", threshold);
    }

    let response = apply_auth(client.post(url))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!(
                "Gagal menghubungi face compare service: {}",
                e
            ))
        })?;

    if !response.status().is_success() {
        return Err(parse_face_service_error(response).await);
    }

    let parsed: VerifyResponse = response.json().await.map_err(|e| {
        AppError::AnyhowError(anyhow::anyhow!(
            "Gagal parse respons /verify-embedding face compare service: {}",
            e
        ))
    })?;

    Ok(FaceVerifyResult {
        is_match: parsed.is_match,
        similarity: parsed.similarity,
        probe_embedding: parsed.probe_face.and_then(|face| face.embedding),
    })
}

#[allow(dead_code)]
pub async fn verify_images(
    reference_bytes: Vec<u8>,
    probe_bytes: Vec<u8>,
) -> Result<FaceVerifyResult, AppError> {
    let client = reqwest::Client::new();
    let url = format!("{}/verify", base_url()?);
    let mut form = Form::new()
        .part(
            "reference_image",
            image_part(reference_bytes, "referensi.jpg")?,
        )
        .part("probe_image", image_part(probe_bytes, "absensi.jpg")?);

    if let Some(threshold) = threshold_from_env() {
        form = form.text("threshold", threshold);
    }

    let response = apply_auth(client.post(url))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            AppError::AnyhowError(anyhow::anyhow!(
                "Gagal menghubungi face compare service: {}",
                e
            ))
        })?;

    if !response.status().is_success() {
        return Err(parse_face_service_error(response).await);
    }

    let parsed: VerifyResponse = response.json().await.map_err(|e| {
        AppError::AnyhowError(anyhow::anyhow!(
            "Gagal parse respons /verify face compare service: {}",
            e
        ))
    })?;

    Ok(FaceVerifyResult {
        is_match: parsed.is_match,
        similarity: parsed.similarity,
        probe_embedding: parsed.probe_face.and_then(|face| face.embedding),
    })
}
