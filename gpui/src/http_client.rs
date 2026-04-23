use futures::future::BoxFuture;
use gpui::http_client::{AsyncBody, HttpClient, Request, Response, Url};
use http::HeaderValue;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::Semaphore;

// Cap simultaneous image fetches so rockboxd doesn't exhaust its file descriptor limit
// when a large album/artist grid triggers many concurrent requests.
const MAX_CONCURRENT_REQUESTS: usize = 8;

pub struct ReqwestHttpClient {
    client: reqwest::Client,
    handle: Handle,
    semaphore: Arc<Semaphore>,
}

impl ReqwestHttpClient {
    pub fn new(handle: Handle) -> Arc<Self> {
        Arc::new(Self {
            client: reqwest::Client::builder()
                .pool_max_idle_per_host(4)
                .build()
                .unwrap_or_default(),
            handle,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS)),
        })
    }
}

impl HttpClient for ReqwestHttpClient {
    fn type_name(&self) -> &'static str {
        "ReqwestHttpClient"
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        None
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let (tx, rx) = tokio::sync::oneshot::channel::<anyhow::Result<Response<AsyncBody>>>();
        let client = self.client.clone();
        let semaphore = self.semaphore.clone();
        self.handle.spawn(async move {
            let _permit = semaphore.acquire().await;
            let _ = tx.send(do_fetch(client, req).await);
        });
        Box::pin(async move {
            rx.await
                .map_err(|_| anyhow::anyhow!("http client channel closed"))?
        })
    }
}

async fn do_fetch(
    client: reqwest::Client,
    req: Request<AsyncBody>,
) -> anyhow::Result<Response<AsyncBody>> {
    let (parts, _body) = req.into_parts();
    let url = parts.uri.to_string();
    let method = reqwest::Method::from_bytes(parts.method.as_str().as_bytes())
        .map_err(|e| anyhow::anyhow!("invalid method: {e}"))?;

    let mut rb = client.request(method, url);
    for (name, value) in &parts.headers {
        rb = rb.header(name, value);
    }

    let resp = rb.send().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let status = resp.status();
    let mut builder = Response::builder().status(status);
    {
        let hm = builder.headers_mut().unwrap();
        for (name, value) in resp.headers() {
            hm.insert(name.clone(), value.clone());
        }
    }
    let bytes = resp.bytes().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(builder
        .body(AsyncBody::from_bytes(bytes))
        .map_err(|e| anyhow::anyhow!("response build: {e}"))?)
}
