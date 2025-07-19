use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
enum ParseError {
    FieldMissing(String),
    UnknownFieldType { current: String, expected: String },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::FieldMissing(field) => write!(f, "Missing field: {}", field),
            ParseError::UnknownFieldType { current, expected } => write!(
                f,
                "Unknown field type: {} (expected: {})",
                current, expected
            ),
        }
    }
}

impl std::error::Error for ParseError {}

trait Parser
where
    Self: Sized,
{
    fn parse(url: &Url) -> Result<Self, ParseError>;
}

#[derive(Debug)]
enum ProxyConfig {
    Vmess(Vmess),
    Vless(Vless),
    Shadowsocks(Shadowsocks),
    Trojan(Trojan),
}

impl ProxyConfig {
    fn id(&self) -> Option<&str> {
        // piski
        match self {
            ProxyConfig::Vless(value) => Some(&value.id),
            ProxyConfig::Vmess(value) => Some(&value.id),
            ProxyConfig::Trojan(value) => Some(&value.id),
            ProxyConfig::Shadowsocks(_) => None,
        }
    }

    fn address(&self) -> &str {
        match self {
            ProxyConfig::Vless(value) => &value.address,
            ProxyConfig::Vmess(value) => &value.address,
            ProxyConfig::Trojan(value) => &value.address,
            ProxyConfig::Shadowsocks(value) => &value.address,
        }
    }

    fn port(&self) -> u16 {
        match self {
            ProxyConfig::Vless(value) => value.port,
            ProxyConfig::Vmess(value) => value.port,
            ProxyConfig::Trojan(value) => value.port,
            ProxyConfig::Shadowsocks(value) => value.port,
        }
    }
}

#[derive(Debug)]
struct Vmess {
    id: String,
    address: String,
    port: u16,
    aid: u32,
    network: String,
    type_field: Option<String>,
    host: Option<String>,
    path: Option<String>,
    tls: bool,
    name: Option<String>,
    // raw parameters store
    extras: HashMap<String, String>,
}

#[derive(Debug)]
struct Vless {
    id: String,
    address: String,
    port: u16,
    security: Option<String>,
    encryption: Option<String>,
    network: String,
    path: Option<String>,
    host: Option<String>,
    tls: bool,
    name: Option<String>,
    extras: HashMap<String, String>,
}

#[derive(Debug)]
struct Shadowsocks {
    method: String,
    password: String,
    address: String,
    port: u16,
    name: Option<String>,
    extras: HashMap<String, String>,
}

#[derive(Debug)]
struct Trojan {
    id: String,
    password: String,
    address: String,
    port: u16,
    sni: Option<String>,
    ws_path: Option<String>,
    host: Option<String>,
    allow_insecure: bool,
    name: Option<String>,
    extras: HashMap<String, String>,
}

impl Parser for Vless {
    fn parse(url: &Url) -> Result<Self, ParseError> {
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        let mut extras = query.clone();
        extras.remove("encryption");
        extras.remove("security");

        let id = url.username().to_string();
        if id.is_empty() {
            return Err(ParseError::FieldMissing("id".to_string()));
        }

        let address = url
            .host_str()
            .ok_or(ParseError::FieldMissing("address".to_string()))?
            .to_string();

        let port = url
            .port()
            .ok_or(ParseError::FieldMissing("port".to_string()))?;

        let network = query
            .get("type")
            .ok_or(ParseError::FieldMissing("network".to_string()))?
            .to_string();

        Ok(Vless {
            id,
            address,
            port,
            network,
            security: query.get("security").cloned(),
            encryption: query.get("encryption").cloned(),
            path: query.get("path").cloned(),
            host: query.get("host").cloned(),

            name: url.fragment().map(|s| s.to_string()),
            extras,

            tls: query.get("security").map(|s| s == "tls").unwrap_or(false),
        })
    }
}

// impl Parser for Vmess {
//     fn parse(url: Url) -> std::io::Result<ProxyConfig> {
//         let payload = &line[8..];
//         let decoded = base64::decode(payload).ok();
//         let json: serde_json::Value = serde_json::from_slice(&decoded).ok();
//         let mut extras = HashMap::new();
//         for (k, v) in json.as_object()?.iter() {
//             if ["add", "port", "id", "aid", "net", "tls", "ps"].contains(&k.as_str()) {
//                 continue;
//             }
//             extras.insert(k.clone(), v.as_str().unwrap_or_default().to_string());
//         }
//         ProxyConfig::Vmess(Vmess {
//             address: json["add"].as_str()?.to_string(),
//             port: json["port"].as_str()?.parse().ok()?,
//             id: json["id"].as_str()?.to_string(),
//             aid: json["aid"].as_str()?.parse().unwrap_or(0),
//             network: json["net"].as_str()?.to_string(),
//             type_field: json
//                 .get("type")
//                 .and_then(|v| v.as_str().map(|s| s.to_string())),
//             host: json
//                 .get("host")
//                 .and_then(|v| v.as_str().map(|s| s.to_string())),
//             path: json
//                 .get("path")
//                 .and_then(|v| v.as_str().map(|s| s.to_string())),
//             tls: json["tls"].as_bool().unwrap_or(false),
//             name: json
//                 .get("ps")
//                 .and_then(|v| v.as_str().map(|s| s.to_string())),
//             extras,
//         })
//     }
// }

// impl Parser for Vless {
//     fn parse(url: Url) -> std::io::Result<ProxyConfig> {
//         let url = Url::parse(line).ok()?;
//         let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
//         let mut extras = query.clone();
//         extras.remove("security");
//         extras.remove("encryption");

//         ProxyConfig::Vless(Vless {
//             address: url.host_str()?.to_string(),
//             port: url.port()?,
//             id: url.username().to_string(),
//             security: query.get("security").cloned(),
//             encryption: query.get("encryption").cloned(),
//             network: query.get("type").cloned().unwrap_or_else(|| "tcp".into()),
//             path: query.get("path").cloned(),
//             host: query.get("host").cloned(),
//             tls: query.get("security").map(|s| s == "tls").unwrap_or(false),
//             name: url.fragment().map(|s| s.to_string()),
//             extras,
//         })
//     }
// }

// impl Parser for Shadowsocks {
//     fn parse(url: Url) -> std::io::Result<ProxyConfig> {
//         let after = &line[5..];
//         let (creds, rest) = after.split_once('@')?;
//         let (method, password) = creds.split_once(':')?;
//         let (addrpart, namepart) = rest.split_once('#').unwrap_or((rest, ""));
//         let (address, port) = addrpart.split_once(':')?;
//         let port = port.parse().ok()?;
//         let mut extras = HashMap::new();

//         ProxyConfig::Shadowsocks(Shadowsocks {
//             method: method.to_string(),
//             password: password.to_string(),
//             address: address.to_string(),
//             port,
//             name: if namepart.is_empty() {
//                 None
//             } else {
//                 Some(namepart.to_string())
//             },
//             extras,
//         })
//     }
// }

// impl Parser for Trojan {
//     fn parse(url: Url) -> Result<Self, ParseError> {
//         let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
//         let mut extras = query.clone();

//         ProxyConfig::Trojan(Trojan {
//             password: url.username().to_string(),
//             address: url.host_str()?.to_string(),
//             port: url.port()?,
//             sni: query.get("sni").cloned(),
//             ws_path: query.get("path").cloned(),
//             host: query.get("host").cloned(),
//             allow_insecure: query
//                 .get("allowInsecure")
//                 .map(|v| v == "1")
//                 .unwrap_or(false),
//             name: url.fragment().map(|s| s.to_string()),
//             extras,
//         })
//     }
// }

fn parse_line(line: &str) -> Result<ProxyConfig, String> {
    let url = Url::parse(line).unwrap();

    match url.scheme() {
        "vless" => Vless::parse(&url)
            .map(ProxyConfig::Vless)
            .map_err(|err| format!("{}", err)),
        other => Err(format!("unknown url scheme: \"{other}\"")),
    }
}

pub fn work(payload: String) {
    let mut configs = Vec::new();

    for line in payload.lines().take(20) {
        println!("{}", line);
        match parse_line(line) {
            Ok(parsed) => configs.push(parsed),
            Err(err) => println!("failed to parse line: {}", err),
        }
    }

    for cfg in configs.iter().take(20) {
        println!("{}", cfg.address())
    }
}
