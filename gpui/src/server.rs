use std::sync::{Arc, LazyLock, RwLock};

#[derive(Clone, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub host: String,
    pub grpc_port: u16,
    pub graphql_port: u16,
    pub http_port: u16,
}

impl ServerInfo {
    pub fn localhost() -> Self {
        Self {
            name: "localhost".to_string(),
            host: "127.0.0.1".to_string(),
            grpc_port: 6061,
            graphql_port: 6062,
            http_port: 6063,
        }
    }

    pub fn grpc_url(&self) -> String {
        format!("http://{}:{}", self.host, self.grpc_port)
    }

    pub fn graphql_url(&self) -> String {
        format!("http://{}:{}", self.host, self.graphql_port)
    }

    pub fn http_url(&self) -> String {
        format!("http://{}:{}", self.host, self.http_port)
    }

    pub fn display_name(&self) -> String {
        if self.host == "127.0.0.1" || self.host == "localhost" {
            "localhost".to_string()
        } else if !self.name.is_empty() && self.name != self.host {
            format!("{} ({})", self.name, self.host)
        } else {
            self.host.clone()
        }
    }

    pub fn is_localhost(&self) -> bool {
        self.host == "127.0.0.1" || self.host == "localhost"
    }
}

static CURRENT_SERVER: LazyLock<RwLock<ServerInfo>> =
    LazyLock::new(|| RwLock::new(ServerInfo::localhost()));

static SERVER_NOTIFY: LazyLock<Arc<tokio::sync::Notify>> =
    LazyLock::new(|| Arc::new(tokio::sync::Notify::new()));

pub fn get_grpc_url() -> String {
    CURRENT_SERVER.read().unwrap().grpc_url()
}

pub fn get_http_url() -> String {
    CURRENT_SERVER.read().unwrap().http_url()
}

pub fn get_covers_base() -> String {
    let s = CURRENT_SERVER.read().unwrap();
    format!("http://{}:{}/covers/", s.host, s.graphql_port)
}

pub fn set_server(info: ServerInfo) {
    *CURRENT_SERVER.write().unwrap() = info;
    SERVER_NOTIFY.notify_waiters();
}

pub fn current_server() -> ServerInfo {
    CURRENT_SERVER.read().unwrap().clone()
}

/// Returns a handle to the server-switch notification.
/// Callers can `.await` `.notified()` to be woken immediately when the active server changes.
pub fn server_notify() -> Arc<tokio::sync::Notify> {
    SERVER_NOTIFY.clone()
}

/// Blocking mDNS scan — returns all discovered rockboxd instances within `timeout`.
/// Looks for `_rockbox._tcp.local.` services; service names prefixed with `grpc-`,
/// `graphql-`, or `http-` update the corresponding port for that host.
pub fn scan_mdns(timeout: std::time::Duration) -> Vec<ServerInfo> {
    use mdns_sd::{ServiceDaemon, ServiceEvent};
    use std::collections::HashMap;

    let mdns = match ServiceDaemon::new() {
        Ok(m) => m,
        Err(_) => return vec![],
    };
    let receiver = match mdns.browse("_rockbox._tcp.local.") {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let mut by_host: HashMap<String, ServerInfo> = HashMap::new();
    let deadline = std::time::Instant::now() + timeout;

    loop {
        let now = std::time::Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline - now;
        let poll = remaining.min(std::time::Duration::from_millis(100));

        match receiver.recv_timeout(poll) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                // Prefer an IPv4 address; only fall back to the hostname when none is present.
                // Hostnames like `foo.local` may resolve to an IPv6 link-local address, which
                // tonic's http2 transport rejects or connects to the wrong interface.
                // get_addresses() returns &HashSet<Ipv4Addr> — all entries are IPv4.
                // Prefer the raw IP over the .local hostname to avoid IPv6 resolution.
                let host = info
                    .get_addresses()
                    .iter()
                    .next()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| info.get_hostname().trim_end_matches('.').to_string());
                let port = info.get_port();
                let fullname = info.get_fullname().to_string();

                let entry = by_host.entry(host.clone()).or_insert_with(|| ServerInfo {
                    name: host.clone(),
                    host: host.clone(),
                    grpc_port: 6061,
                    graphql_port: 6062,
                    http_port: 6063,
                });

                if fullname.starts_with("grpc-") {
                    entry.grpc_port = port;
                } else if fullname.starts_with("graphql-") {
                    entry.graphql_port = port;
                } else if fullname.starts_with("http-") {
                    entry.http_port = port;
                }
            }
            Ok(_) | Err(_) => {}
        }
    }

    // Exclude localhost — handled separately as the priority default.
    by_host
        .into_values()
        .filter(|s| s.host != "127.0.0.1" && s.host != "localhost")
        .collect()
}
