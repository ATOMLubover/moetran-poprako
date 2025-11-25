use std::{cell::LazyCell, collections::HashMap, ops::Deref as _, time::Duration};

use reqwest::header::{self, HeaderName, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};

use tracing::{debug, warn};

// ================== API Client 封装结构 ==================

struct ApiClient {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

impl ApiClient {
    const TIMEOUT_SECS: u64 = 5;

    // new：仅供模块内部懒初始化使用，不对外暴露
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

    // 通用 GET：执行请求 -> 状态检查 -> 解析 JSON
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

    // 通用 POST：构造请求（必要时空 body） -> 附加头 -> 状态检查 -> 解析 JSON
    pub async fn https_post<B, R>(
        client: &reqwest::Client,
        url: reqwest::Url,
        headers: Vec<(HeaderName, HeaderValue)>,
        body: Option<B>,
    ) -> Result<R, String>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        let mut req = client.post(url);

        match body {
            Some(b) => {
                req = req.json(&b);
            }
            None => {
                req = req.body("");
            }
        }

        let mut headers_map = reqwest::header::HeaderMap::new();

        headers.into_iter().for_each(|(key, value)| {
            if let Some(prev) = headers_map.insert(key, value) {
                warn!(
                    ?prev,
                    "Header key duplicated when building headers for POST"
                );
            }
        });

        let resp = req
            .headers(headers_map)
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

    pub static POPRAKO_API_BASE: reqwest::Url = "http://127.0.0.1:8080/api/v1/".parse().expect("invalid POPRAKO_API_BASE URL");

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

    let mut headers = Vec::new();

    if let Some(token) = crate::token::cached_moetran_token() {
        headers.push((
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|err| format!("Invalid token header value: {}", err))?,
        ));
    }

    ApiClient::https_post(&client, url, headers, body).await
}

pub async fn moetran_get<R>(path: &str, query: Option<&HashMap<&str, String>>) -> Result<R, String>
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

    let mut url = base
        .join(path)
        .map_err(|err| format!("Failed to build URL for {}: {}", path, err))?;

    if let Some(q) = query {
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in q.iter() {
                pairs.append_pair(key, value);
            }
        }
        // drop(pairs) happens here when going out of scope
    }

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

    let mut headers = Vec::new();

    if let Some(token) = crate::token::cached_poprako_token() {
        headers.push((
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|err| format!("Invalid token header value: {}", err))?,
        ));
    }

    ApiClient::https_post(&client, url, headers, body).await
}

pub async fn poprako_get<R>(path: &str, query: Option<&HashMap<&str, String>>) -> Result<R, String>
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

    let mut url = base
        .join(path)
        .map_err(|err| format!("Failed to build URL for {}: {}", path, err))?;

    if let Some(q) = query {
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in q.iter() {
                pairs.append_pair(key, value);
            }
        }
    }

    ApiClient::https_get(&client, url).await
}
