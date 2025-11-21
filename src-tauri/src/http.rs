use std::{cell::LazyCell, ops::Deref as _, time::Duration};

use reqwest::header::{self, HeaderName, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};

use tracing::{debug, warn};

struct ApiClient {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

impl ApiClient {
    const TIMEOUT_SECS: u64 = 5;

    /// `new` 不设定为 pub，因为只在本模块内懒初始化时使用
    fn new(base_url: reqwest::Url, default_headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        let mut default_header_map = reqwest::header::HeaderMap::new();

        default_headers.into_iter().for_each(|(key, value)| {
            if let Some(prev) = default_header_map.insert(key, value) {
                warn!(?prev, "Header key duplicated when building headers");
            }
        });

        debug!(?base_url, ?default_header_map, "ApiClient is now building");

        let client = reqwest::Client::builder()
            .default_headers(default_header_map)
            .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
            .build()
            .expect("Failed to build reqwest Client");

        debug!("ApiClient built successfully");

        Self { client, base_url }
    }

    pub async fn https_get<R>(client: &reqwest::Client, url: reqwest::Url) -> Result<R, String>
    where
        R: DeserializeOwned,
    {
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|err| format!("request send error: {}", err))?;

        let resp = resp
            .error_for_status()
            .map_err(|err| format!("http error: {}", err))?;

        let parsed = resp
            .json::<R>()
            .await
            .map_err(|err| format!("json parse error: {}", err))?;

        Ok(parsed)
    }

    pub async fn https_post<B, R>(
        client: &reqwest::Client,
        url: reqwest::Url,
        body: Option<B>,
    ) -> Result<R, String>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        let mut req = client.post(url);

        if let Some(b) = body {
            req = req.json(&b);
        } else {
            // send empty body to match browser behavior for some endpoints
            req = req.body("");
        }

        let resp = req
            .send()
            .await
            .map_err(|err| format!("request send error: {}", err))?;

        let resp = resp
            .error_for_status()
            .map_err(|err| format!("http error: {}", err))?;

        let parsed = resp
            .json::<R>()
            .await
            .map_err(|err| format!("json parse error: {}", err))?;

        Ok(parsed)
    }
}

thread_local! {
    pub static MOETRAN_API_BASE: reqwest::Url = "https://api.moetran.com/v1/".parse().expect("invalid MOETRAN_API_BASE URL");

    static MOETRAN_API_CLIENT: LazyCell<ApiClient> = LazyCell::new(|| {
        let base = MOETRAN_API_BASE.with(|b| b.clone());

        let default_headers = vec![
            // Origin/Referer are sometimes validated; include as defaults here for API calls originating from the app
            (header::ACCEPT, HeaderValue::from_static("application/json, text/plain, */*")),
            (header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")),
            (header::ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN")),
            (header::ORIGIN, HeaderValue::from_static("https://moetran.com")),
            (header::REFERER, HeaderValue::from_static("https://moetran.com/")),
        ];

        ApiClient::new(base, default_headers)
    });

    pub static POPRAKO_API_BASE: reqwest::Url = "https://hatsu1ki-lb-site.com/api/v1/".parse().expect("invalid POPRAKO_API_BASE URL");

    static POPRAKO_API_CLIENT: LazyCell<ApiClient> = LazyCell::new(|| {
        let base = POPRAKO_API_BASE.with(|b| b.clone());

        let default_headers = vec![
            (HeaderName::from_static("accept"), HeaderValue::from_static("application/json, text/plain, */*")),
            (HeaderName::from_static("user-agent"), HeaderValue::from_static("moetran-native-client/1.0")),
        ];

        ApiClient::new(base, default_headers)
    });
}

pub async fn moetran_post_opt<B, R>(path: &str, body: Option<B>) -> Result<R, String>
where
    B: Serialize,
    R: DeserializeOwned,
{
    if path.is_empty() || path.starts_with('/') {
        return Err(format!("Invalid path for moetran_post_opt: {}", path));
    }

    let (client, base) = MOETRAN_API_CLIENT.with(|lazy| {
        let api = lazy.deref();
        (api.client.clone(), api.base_url.clone())
    });

    let url = base
        .join(path)
        .map_err(|err| format!("Failed to build URL for {}: {}", path, err))?;

    ApiClient::https_post(&client, url, body).await
}

pub async fn moetran_get<R>(path: &str) -> Result<R, String>
where
    R: DeserializeOwned,
{
    if path.is_empty() || path.starts_with('/') {
        return Err(format!("Invalid path for moetran_get: {}", path));
    }

    let (client, base) = MOETRAN_API_CLIENT.with(|lazy| {
        let api_client = lazy.deref();
        (api_client.client.clone(), api_client.base_url.clone())
    });

    let url = base
        .join(path)
        .map_err(|err| format!("Failed to build URL for {}: {}", path, err))?;

    ApiClient::https_get(&client, url).await
}

pub async fn poprako_post_opt<B, R>(path: &str, body: Option<B>) -> Result<R, String>
where
    B: Serialize,
    R: DeserializeOwned,
{
    if path.is_empty() || path.starts_with('/') {
        return Err(format!("Invalid path for poprako_post_opt: {}", path));
    }

    let (client, base) = POPRAKO_API_CLIENT.with(|lazy| {
        let api = lazy.deref();
        (api.client.clone(), api.base_url.clone())
    });

    let url = base
        .join(path)
        .map_err(|err| format!("Failed to build URL for {}: {}", path, err))?;

    ApiClient::https_post(&client, url, body).await
}

pub async fn poprako_get<R>(path: &str) -> Result<R, String>
where
    R: DeserializeOwned,
{
    if path.is_empty() || path.starts_with('/') {
        return Err(format!("Invalid path for poprako_get: {}", path));
    }

    let (client, base) = POPRAKO_API_CLIENT.with(|lazy| {
        let api_client = lazy.deref();
        (api_client.client.clone(), api_client.base_url.clone())
    });

    let url = base
        .join(path)
        .map_err(|err| format!("Failed to build URL for {}: {}", path, err))?;

    ApiClient::https_get(&client, url).await
}
