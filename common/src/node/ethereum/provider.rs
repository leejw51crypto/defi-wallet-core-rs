use crate::EthError;
use ethers::providers::{Http, Provider};
use url::Url;

use once_cell::sync::Lazy; // 1.3.1
use std::sync::{Arc, Mutex};

struct HttpClientInfo {
    pub agentinfo: String,
}
impl HttpClientInfo {
    pub fn new() -> Self {
        let useragentinfo = std::env::var("DEFIWALLETCORE_AGENTINFO");
        if let Ok(agentinfo) = useragentinfo {
            HttpClientInfo { agentinfo }
        } else {
            HttpClientInfo {
                agentinfo: "defiwalletcore".to_string(),
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
static G_ETHERS_CLIENT_INFO: Lazy<Arc<Mutex<HttpClientInfo>>> =
    Lazy::new(|| Arc::new(Mutex::new(HttpClientInfo::new())));

#[cfg(not(target_arch = "wasm32"))]
// urlinfo: url string of the node to connect to, "http://mynode:8545"
// agentinfo: agent string for http header
pub async fn get_ethers_provider(urlinfo: &str) -> Result<Provider<Http>, EthError> {
    let url = Url::parse(urlinfo).map_err(EthError::NodeUrl)?;
    let info = G_ETHERS_CLIENT_INFO
        .lock()
        .expect("get HttpClientInfo lock");
    let agentinfo = &info.agentinfo;
    let client = reqwest::Client::builder()
        .user_agent(agentinfo)
        .build()
        .map_err(EthError::ClientError)?;
    let httpprovider = Http::new_with_client(url, client);
    let finalprovider = Provider::new(httpprovider);
    Ok(finalprovider)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_ethers_httpagent(agentinfo: &str) -> Result<bool, EthError> {
    let mut info = G_ETHERS_CLIENT_INFO
        .lock()
        .expect("get HttpClientInfo lock in set_ethers_httpagent");
    info.agentinfo = agentinfo.to_string();
    Ok(true)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_ethers_httpagent() -> Result<String, EthError> {
    let info = G_ETHERS_CLIENT_INFO
        .lock()
        .expect("get HttpClientInfo lock in get_ethers_httpagent");
    Ok(info.agentinfo.clone())
}

#[cfg(target_arch = "wasm32")]
pub async fn get_ethers_provider(urlinfo: &str) -> Result<Provider<Http>, EthError> {
    let url = Url::parse(urlinfo).map_err(EthError: NodeUrl)?;
    let client = reqwest::Client::builder()
        .build()
        .map_err(EthError::ClientError)?;
    let httpprovider = Http::new_with_client(url, client);
    let finalprovider = Provider::new(httpprovider);
    Ok(finalprovider)
}

#[cfg(test)]
mod ethereum_provider_tests {
    use super::*;

    const TEST_AGENTINFO: &str = "testagentinfo";

    #[test]
    fn test_provider_agentinfo() {
        set_ethers_httpagent(TEST_AGENTINFO).unwrap();
        let value = get_ethers_httpagent().unwrap();
        assert_eq!(value, TEST_AGENTINFO);
    }
}
