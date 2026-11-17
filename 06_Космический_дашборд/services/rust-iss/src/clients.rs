use std::time::Duration;
use serde_json::Value;

pub fn new_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .user_agent("StudentSpaceDashboard/1.0")
        .build()?)
}

// Один запрос за раз и пауза ограничивают частоту внутри единственного rust-контейнера.
static GATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn get_json(request: reqwest::RequestBuilder) -> anyhow::Result<Value> {
    let _guard = GATE.lock().await;
    for attempt in 0..3 {
        let response = request.try_clone().ok_or_else(|| anyhow::anyhow!("Request cannot be retried"))?.send().await;
        match response {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    let value = response.json().await?;
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    return Ok(value);
                }
                if !status.is_server_error() && status.as_u16() != 429 {
                    anyhow::bail!("UPSTREAM_{}", status.as_u16());
                }
                if attempt == 2 { anyhow::bail!("UPSTREAM_{}", status.as_u16()); }
            }
            Err(_) if attempt == 2 => anyhow::bail!("UPSTREAM_TIMEOUT"),
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
    }
    unreachable!()
}
