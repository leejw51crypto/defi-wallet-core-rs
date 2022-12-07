use defi_wallet_core_common::abi::EthAbiToken;
use defi_wallet_core_common::abi::EthAbiToken::Uint;
use defi_wallet_core_common::contract::ContractCall;
use defi_wallet_core_common::contract::DynamicContract;
use defi_wallet_core_common::EthAbiTokenBind;
use ethers::abi::Detokenize;
use ethers::abi::Tokenize;
// use hashmap
use std::collections::HashMap;
//use ethers_signers::{MnemonicBuilder, coins_bip39::English};
use anyhow::anyhow;
use anyhow::Result;
use ethers::abi::Abi;
use ethers::abi::InvalidOutputType;
use ethers::abi::Token;
use ethers::contract::ContractFactory;
use ethers::prelude::*;
use ethers::signers::coins_bip39::English;
use ethers::signers::MnemonicBuilder;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::Bytes;
use sha2::Digest;
use std::io::prelude::*;
use std::sync::Arc;
use std::str::FromStr;

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

fn make_wallet(index: i32) -> Result<LocalWallet> {
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
async fn main2() -> Result<()> {
    println!("test");
    let jsonstring =
        std::fs::read_to_string("../contracts/artifacts/contracts/TestERC721.sol/TestERC721.json")?;
    let json = serde_json::from_str::<serde_json::Value>(&jsonstring)?;
    let abi = json["abi"].to_string();
    // create Abi from abi
    let abi: Abi = serde_json::from_str(&abi)?;
    let bytecodestring = json["bytecode"]
        .as_str()
        .ok_or_else(|| anyhow!("no bytecode"))?;
    // skip 2 bytes and hex decode bytecodestring
    let bytecode = hex::decode(&bytecodestring[2..])?;
    // convert bytecode to Bytes
    let bytecode = Bytes::from(bytecode);

    let rpc = std::env::var("MYCRONOSRPC")?;

    let fromwallet = make_wallet(0)?;
    println!("Address: {:?}", fromwallet.address());
    let towallet = make_wallet(2)?;
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
    // print lengh of data
    println!("data length: {}", data.len());
    // sha2 compute hash
    let mut hasher = sha2::Sha256::default();
    hasher.update(&data);
    let hash = hasher.finalize();
    // print hex of hash
    println!("hash: {}", hex::encode(&hash));

    //tx.max_fee_per_gas = self.max_fee_per_gas;
    //  tx.max_priority_fee_per_gas = self.max_priority_fee_per_gas;

    let nonce = client
        .get_transaction_count(fromwallet.address(), None)
        .await?;

    let tx = Eip1559TransactionRequest::new()
        .from(fromwallet.address())
        //.to(towallet.address())
        .data(data)
        // good
        .gas(1000000)
        .max_fee_per_gas(1000000000)
        .max_priority_fee_per_gas(1000000000)
        // error
        //.gas(2194000000u64)
        //.max_fee_per_gas(1000000)
        //.max_priority_fee_per_gas(1000000)
        .chain_id(1)
        .nonce(nonce)
        .value(0u64);
    // convertr tx to TypedTransaction
    let tx: TypedTransaction = tx.try_into()?;
    // debug print tx
    println!("tx: {:?}", tx);
    let sig = fromwallet.sign_transaction(&tx).await?;
    // debug print sig
    println!("sig: {:?}", sig);
    let signed_tx = tx.rlp_signed(&sig).clone();

    // write signed_tx to file
    let mut file = std::fs::File::create("signed_tx.bin")?;
    // write bytes to file
    file.write_all(&signed_tx)?;
    // print lnegth of signed_tx
    println!("signed_tx length: {}", signed_tx.len());

    //return Ok(());
    let pending_tx = client.send_raw_transaction(signed_tx).await?;
    //let txhash = client.send_transaction(&signed_tx, None).await?;
    println!("txhash: {:?}", pending_tx);
    let receipt = pending_tx.await.unwrap().unwrap();
    println!("receipt: {:?}", receipt);

    // wait for confirmation
    //let receipt = client.wait_for_transaction_receipt(txhash, None, None).await?;
    println!("receipt: {:?}", receipt);

    Ok(())
}

/*
 pub enum EthAbiTokenBind {
        Address { data: String },
        FixedBytes { data: Vec<u8> },
        Bytes { data: Vec<u8> },
        Int { data: String },
        Uint { data: String },
        Bool { data: bool },
        Str { data: String },
        FixedArray { data: Vec<EthAbiTokenBind> },
        Array { data: Vec<EthAbiTokenBind> },
        Tuple { data: Vec<EthAbiTokenBind> },
    }
*/
#[tokio::main]
async fn main() -> Result<()> {
    let mut tokens: Vec<EthAbiTokenBind> = vec![];
    let datum= EthAbiTokenBind::Int { data: "1".to_string() };

    tokens.push(EthAbiTokenBind::Address {
        data: "0x0000000000000000000000000000000000000000".to_string(),
    });
    tokens.push(EthAbiTokenBind::FixedBytes { data: vec![1,2] });
    tokens.push(EthAbiTokenBind::Bytes { data: vec![1,2] });
    tokens.push(EthAbiTokenBind::Int { data: "1".to_string() });
    tokens.push(EthAbiTokenBind::Uint { data: "1".to_string() });
    tokens.push(EthAbiTokenBind::Bool { data: true });
    tokens.push(EthAbiTokenBind::Str { data: "test".to_string() });
    tokens.push(EthAbiTokenBind::FixedArray { data: vec![datum] });
    tokens.push(EthAbiTokenBind::Array { data: vec![datum] });
    tokens.push(EthAbiTokenBind::Tuple { data: vec![datum] });


    


    
    

    // make json from tokens
    let json = serde_json::to_string(&tokens)?;
    //let json = serde_json::to_string_pretty(&tokens)?;
    
    // print json
    println!("json: {}", json);
    
    
    println!("test");
    Ok(())
}
