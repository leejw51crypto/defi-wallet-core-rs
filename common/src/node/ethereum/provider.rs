use crate::EthError;
use ethers::providers::{Http, Provider};
use url::Url;

static DEFIWALLETCORE_AGENTINFO: Option<&'static str> = option_env!("DEFIWALLETCORE_AGENTINFO");

// urlinfo: url string of the node to connect to, "http://mynode:8545"
// agentinfo: agent string for http header
pub async fn get_ethers_provider(urlinfo: &str) -> Result<Provider<Http>, EthError> {
    let url = Url::parse(urlinfo).unwrap();
    let mut agentinfo = "defiwalletcore";
    if let Some(v) = DEFIWALLETCORE_AGENTINFO {
        agentinfo = v;
    }
    let client = reqwest::Client::builder()
        .user_agent(agentinfo)
        .build()
        .map_err(EthError::ClientError)?;
    let httpprovider = Http::new_with_client(url, client);
    let finalprovider = Provider::new(httpprovider);
    Ok(finalprovider)
}
