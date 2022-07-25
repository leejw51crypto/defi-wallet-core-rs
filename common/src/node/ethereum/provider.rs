use crate::EthError;
use ethers::providers::{Http, Provider};
use url::Url;

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::OnceCell; // 1.3.1

#[cfg(not(target_arch = "wasm32"))]
static AGENT_INFO: OnceCell<String> = OnceCell::new();

// urlinfo: url string of the node to connect to, "http://mynode:8545"
// agentinfo: agent string for http header
pub async fn get_ethers_provider(urlinfo: &str) -> Result<Provider<Http>, EthError> {
    let url = Url::parse(urlinfo).map_err(EthError::NodeUrl)?;
    #[cfg(target_arch = "wasm32")]
    let client = reqwest::Client::builder()
        .build()
        .map_err(EthError::ClientError)?;
    #[cfg(not(target_arch = "wasm32"))]
    let client = {
        let agentinfo: &String = AGENT_INFO.get_or_init(|| {
            std::env::var("DEFIWALLETCORE_AGENTINFO")
                .unwrap_or_else(|_| "defiwalletcore".to_string())
        });
        reqwest::Client::builder()
            .user_agent(agentinfo)
            .build()
            .map_err(EthError::ClientError)?
    };

    let httpprovider = Http::new_with_client(url, client);
    let finalprovider = Provider::new(httpprovider);
    Ok(finalprovider)
}
