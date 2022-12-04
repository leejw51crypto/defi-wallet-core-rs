use defi_wallet_core_common::abi::EthAbiToken;
use defi_wallet_core_common::abi::EthAbiToken::Uint;
use defi_wallet_core_common::contract::ContractCall;
use defi_wallet_core_common::contract::DynamicContract;
use defi_wallet_core_common::EthAbiTokenBind;
use ethers::abi::Detokenize;
use ethers::abi::Tokenize;
//use ethers_signers::{MnemonicBuilder, coins_bip39::English};
use anyhow::anyhow;
use anyhow::Result;
use ethers::abi::InvalidOutputType;
use ethers::abi::Token;
use ethers::prelude::*;
use ethers::signers::coins_bip39::English;
use ethers::signers::MnemonicBuilder;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::sync::Arc;
use ethers::contract::ContractFactory;
use ethers::abi::Abi;
use ethers::types::Bytes;

fn encode_deploy_contract(
    rpcserver: String,
    abi: String,
    bytecode: Vec<u8>,
    function_args: String,
) -> Result<Vec<u8>> {
    let params: Vec<EthAbiTokenBind> = serde_json::from_str(&function_args)?;
    let mut tokens: Vec<Token> = vec![];
    for param in params {
        let abitoken = EthAbiToken::try_from(&param)?;
        let token = Token::try_from(&abitoken)?;
        tokens.push(token);
    }
    let client: Provider<Http> = Provider::<Http>::try_from(&rpcserver)?;
    let bytecode = Bytes::from(bytecode);
    let abi: ethers::abi::Abi = serde_json::from_str(&abi)?;
    let client = Arc::new(client);
    let factory = ContractFactory::new(abi, bytecode, client);
    let deployer = factory
        .deploy_tokens(tokens)
        .map_err(|e| anyhow!(e.to_string()))?;
    let data = deployer.tx.data().ok_or_else(|| anyhow!("no data"))?;
    let data = data.to_vec();

    Ok(data)
}

fn make_wallet(index:i32) -> Result<LocalWallet> {
    let mnemonics: PathOrString = std::env::var("MYMNEMONICS")?.as_str().into();
    // format string
    let my_path = format!("m/44'/60'/0'/0/{}", index);
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonics)
        .derivation_path(&my_path)?
        .build()?;
    
    Ok(wallet)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("test");
    let jsonstring =
        std::fs::read_to_string("../contracts/artifacts/contracts/TestERC721.sol/TestERC721.json")?;
    let json= serde_json::from_str::<serde_json::Value>(&jsonstring)?;
    let abi = json["abi"].to_string();
    // create Abi from abi
    let abi: Abi = serde_json::from_str(&abi)?;
    let bytecodestring=json["bytecode"].as_str().ok_or_else(|| anyhow!("no bytecode"))?;
    // skip 2 bytes and hex decode bytecodestring
    let bytecode = hex::decode(&bytecodestring[2..])?;
    // convert bytecode to Bytes
    let bytecode = Bytes::from(bytecode);
    

    let rpc = std::env::var("MYCRONOSRPC")?;


    let fromwallet= make_wallet(0)?;
    println!("Address: {:?}", fromwallet.address());
    let towallet= make_wallet(2)?;
    println!("Address: {:?}", towallet.address());

    let mut tokens: Vec<Token> = vec![];
    let client: Provider<Http> = Provider::<Http>::try_from(&rpc)?;
    let myclient = Arc::new(client.clone());
    let factory = ContractFactory::new(abi, bytecode, myclient);
    let deployer = factory
        .deploy_tokens(tokens)
        .map_err(|e| anyhow!(e.to_string()))?;
    let data = deployer.tx.data().ok_or_else(|| anyhow!("no data"))?;
    let data = data.to_vec();

    let tx = Eip1559TransactionRequest::new()
        .from(fromwallet.address())
        .to(towallet.address())
        .gas(1000000)
        .data(data)
        .value(0u64);
    // convertr tx to TypedTransaction
    let tx:TypedTransaction = tx.try_into()?;
    let sig = fromwallet.sign_transaction(&tx).await?;
    let signed_tx = tx.rlp_signed(&sig).clone();
    let txhash = client.send_raw_transaction(signed_tx).await?;
    //let txhash = client.send_transaction(&signed_tx, None).await?;
    println!("txhash: {:?}", txhash);

    

    Ok(())
}
