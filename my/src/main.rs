use defi_wallet_core_common::abi::EthAbiToken::Uint;
use defi_wallet_core_common::contract::ContractCall;
use defi_wallet_core_common::contract::DynamicContract;
use defi_wallet_core_common::EthAbiTokenBind;
use ethers::abi::Detokenize;
use ethers::abi::Tokenize;

use anyhow::Result;
use ethers::abi::InvalidOutputType;
use ethers::abi::Token;
use ethers::prelude::*;
// use Token

#[derive(Debug)]
pub struct MyDetokenizer {
    json: String,
}

#[derive(Debug)]
pub struct MyTokenizer {
    json: String,
}

impl Tokenize for MyTokenizer {
    fn into_tokens(self) -> Vec<Token> {
        let tokens: Vec<Token> = serde_json::from_str(&self.json).unwrap();
        // debug print tokens
        println!("Tokenize into_tokens tokens: {:?}", tokens);
        tokens
    }
}

impl Detokenize for MyDetokenizer {
    fn from_tokens(tokens: Vec<Token>) -> std::result::Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        println!("MyDetokenizer::from_tokens {:?}", tokens);
        let json = serde_json::to_string(&tokens)
            .map_err(|e| InvalidOutputType(format!("serde json error {:?}", e,)))?;
        Ok(MyDetokenizer { json })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let abi_json = std::fs::read_to_string("../common/src/contract/erc721-abi.json")?;
    let contract_address = std::env::var("MYCONTRACT721")?;
    let rpc = std::env::var("MYCRONOSRPC")?;

    let client = Provider::<Http>::try_from(rpc)?;
    let contract = DynamicContract::new(&contract_address, &abi_json, client)?;
    let params = vec![EthAbiTokenBind::Uint {
        data: "1".to_string(),
    }];
    // [{"Uint":{"data":"1"}}]
    // json encoding of params
    let json = serde_json::to_string(&params)?;
    // print json
    println!("json: {}", json);
    let params2: Vec<EthAbiTokenBind> = serde_json::from_str(&json)?;
    let mycall: ContractCall<_, MyDetokenizer> = contract.function_call2("ownerOf", params2)?;
    let feedback = mycall.call().await?;
    println!("mycall ok");
    // debug print feedback
    println!("feedback: {:?}", feedback);

    Ok(())
}

#[tokio::main]
async fn main2() -> Result<()> {
    let abi_json = std::fs::read_to_string("../common/src/contract/erc721-abi.json")?;
    let contract_address = std::env::var("MYCONTRACT721")?;
    let rpc = std::env::var("MYCRONOSRPC")?;

    let client = Provider::<Http>::try_from(rpc)?;
    let contract = DynamicContract::new(&contract_address, &abi_json, client)?;
    let tokens: Vec<Token> = vec![Token::Uint(1.into())];
    let tokensjson = serde_json::to_string(&tokens)?;
    let tokens2: Vec<Token> = serde_json::from_str(&tokensjson)?;
    println!("tokens json: {:?}", tokens);
    println!("tokens2 json: {:?}", tokens2);

    let mydata = MyTokenizer { json: tokensjson };

    let tokens = Token::Uint(1.into());
    let json = serde_json::to_string(&tokens)?;
    println!("tokens: {:?}", tokens);
    println!("json: {}", json);

    //return Ok(());

    let params = MyTokenizer {
        json: json.to_string(),
    };
    let mycontract: ContractCall<_, MyDetokenizer> = contract.function_call("ownerOf", mydata)?;
    let feedback = mycontract.call().await?;
    println!("mycall ok");
    // debug print feedback
    println!("feedback: {:?}", feedback);

    Ok(())
}
