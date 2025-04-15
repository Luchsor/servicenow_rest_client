#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::StatusCode;
use serde::Serialize;
use std::{thread, time::Duration};
use tokio::time::sleep;
use crate::auth::{Authenticator, TokenAuth};

pub struct SNClient {
    url: String,
    client: Client,
    max_retries: usize,
}
impl SNClient {
    pub fn new(url: &str, max_retries: usize) -> Self {
        let client = Client::new();
        SNClient {
            url: url.to_string(),
            client: client,
            max_retries: max_retries,
        }
    }

    fn calculate_delay(&self, attempts: usize) -> Duration {
        let base_delay_secs = 1;
        let delay_secs = base_delay_secs * 2u64.pow(attempts.saturating_sub(1) as u32);
        Duration::from_secs(delay_secs)
    }

    async fn execute_with_retry<F>(&self, build_request: F) -> Result<Response, reqwest::Error>
    where
        F: Fn() -> RequestBuilder, // Closure that builds the specific request
    {
        let mut attempts = 0;

        loop {
            let request_builder = build_request();
            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();

                    // 2xx success codes
                    if status.is_success() {
                        return Ok(response);
                    }

                    // 5xx server error codes
                    // Retry logic for server errors
                    if status.is_server_error() && attempts < self.max_retries {
                        attempts += 1;
                        let delay = self.calculate_delay(attempts);
                        sleep(delay);
                        continue; 
                    }

                    // 4xx or other non-retryable errors
                    match response.error_for_status() {
                        Ok(_) => unreachable!(),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.max_retries || !(e.is_connect() || e.is_timeout()) {
                        return Err(e); 
                    }
                    let delay = self.calculate_delay(attempts);
                    sleep(delay);
                }
            }
        }
    }

    pub async fn get(&self, endpoint: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}", self.url, endpoint);
        let response = self.execute_with_retry(|| self.client.get(&url)).await?;
        response.text().await
    }

    pub async fn post<T: Serialize + Send + Sync>(&self, endpoint: &str, body: &T) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}", self.url, endpoint);
        let response = self.execute_with_retry(|| self.client.post(&url).json(body)).await?;
        response.text().await
    }

    pub async fn put<T: Serialize + Send + Sync>(&self, endpoint: &str, body: &T) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}", self.url, endpoint);
        let response = self.execute_with_retry(|| self.client.put(&url).json(body)).await?;
        response.text().await
    }

    pub async fn patch<T: Serialize + Send + Sync>(&self, endpoint: &str, body: &T) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}", self.url, endpoint);
        let response = self.execute_with_retry(|| self.client.patch(&url).json(body)).await?;
        response.text().await
    }

    pub async fn delete(&self, endpoint: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}", self.url, endpoint);
        let response = self.execute_with_retry(|| self.client.delete(&url)).await?;
        response.text().await
    }

}

#[cfg(feature = "python-bindings")]
#[pymodule]
fn sn_client(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SNClient>()?;
    Ok(())
}