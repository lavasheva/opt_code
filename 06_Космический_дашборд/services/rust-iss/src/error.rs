use axum::{Json, response::{IntoResponse, Response}};

pub struct ApiError { code: String, trace_id: String }

impl ApiError {
    pub fn internal(error: impl std::fmt::Display) -> Self {
        let detail = error.to_string();
        let code = if detail.starts_with("UPSTREAM_") { detail.clone() } else { "INTERNAL_ERROR".into() };
        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let trace_id = format!("{}-{}", chrono::Utc::now().timestamp_micros(),
            SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        // Не пишем URL с ключами и строки подключения в логи.
        tracing::error!(%trace_id, %code, "Request failed");
        Self { code, trace_id }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        Json(serde_json::json!({"ok":false,"error":{
            "code":self.code,"message":"Не удалось получить данные", "trace_id":self.trace_id
        }})).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn business_error_uses_http_200() {
        let response = ApiError::internal("UPSTREAM_403").into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
    #[test]
    fn internal_details_do_not_become_error_code() {
        let error = ApiError::internal("database password must not leak");
        assert_eq!(error.code, "INTERNAL_ERROR");
    }
}
