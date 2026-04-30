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
/// Rank an IPv4 address by how likely it is to be a real local-network address.
/// Lower is better: 192.168.x.x → 0, 10.x.x.x → 1, others (172.x.x.x Docker bridges) → 2.
fn addr_preference(a: &std::net::Ipv4Addr) -> u8 {
    let o = a.octets();
    if o[0] == 192 && o[1] == 168 {
        0
    } else if o[0] == 10 {
        1
    } else {
        2
    }
}

/// Pick the most-preferred non-loopback, non-link-local IPv4 address from a set.
/// Prefers home-network ranges (192.168.x.x, 10.x.x.x) over Docker/VM bridge ranges
/// (172.16.x.x – 172.31.x.x) so that all service records for the same physical host
/// always resolve to the same key and get merged into a single ServerInfo entry.
fn best_addr(addrs: &std::collections::HashSet<std::net::Ipv4Addr>) -> Option<String> {
    let mut candidates: Vec<std::net::Ipv4Addr> = addrs
        .iter()
        .filter(|a| !a.is_loopback() && !a.is_link_local())
        .cloned()
        .collect();
    candidates.sort_by_key(addr_preference);
    candidates.into_iter().next().map(|a| a.to_string())
}

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
                // Always pick the best-ranked address so that grpc-, graphql-, and http-
                // records for the same physical host all hash to the same key.
                let host = best_addr(info.get_addresses())
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
