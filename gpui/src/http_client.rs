use futures::future::BoxFuture;
use gpui::http_client::{AsyncBody, HttpClient, Request, Response, Url};
use http::HeaderValue;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct ReqwestHttpClient {
    client: reqwest::Client,
    handle: Handle,
}

impl ReqwestHttpClient {
    pub fn new(handle: Handle) -> Arc<Self> {
        Arc::new(Self {
            client: reqwest::Client::new(),
            handle,
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
        // Bridge tokio (reqwest) and GPUI's smol executor via a tokio oneshot channel.
        // tokio::sync::oneshot::Receiver implements Future using standard Waker, so it can
        // be awaited from any executor context — including GPUI's smol background executor.
        let (tx, rx) = tokio::sync::oneshot::channel::<anyhow::Result<Response<AsyncBody>>>();
        let client = self.client.clone();
        self.handle.spawn(async move {
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
