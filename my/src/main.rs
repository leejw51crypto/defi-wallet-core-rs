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
    let mnemonics = std::env::var("MYMNEMONICS")?;
    let rpc = std::env::var("MYCRONOSRPC")?;
    let myfromaddress = std::env::var("MYFROMADDRESS")?;
    let mytoaddress = std::env::var("MYTOADDRESS")?;
    // read token id from user
    let mut token_id: String = "1".into();
    //println!("Enter token id:");
    // std::io::stdin().read_line(&mut token_id)?;

    let client = Provider::<Http>::try_from(rpc)?;
    // make dummy client
    let contract = DynamicContract::new(&contract_address, &abi_json, client)?;

    let params = vec![
        EthAbiTokenBind::Address {
            data: myfromaddress,
        },
        EthAbiTokenBind::Address { data: mytoaddress },
        EthAbiTokenBind::Uint { data: token_id },
    ];

    let json = serde_json::to_string(&params)?;
    // [{"Address":{"data":"0x...."}},{"Address":{"data":"0x...."}},{"Uint":{"data":"\\5\n"}}]
    // print json
    println!("json: {}", json);
    //safeTransferFrom , ownerOf
    let mycall: ContractCall<_, MyDetokenizer> =
        contract.function_call("safeTransferFrom", params)?;
    let tx = mycall.get_tx();
    println!("tx: {:?}", tx);

    Ok(())
}

async fn test_call() -> Result<()> {
    let abi_json = std::fs::read_to_string("../common/src/contract/erc721-abi.json")?;
    let contract_address = std::env::var("MYCONTRACT721")?;
    let rpc = std::env::var("MYCRONOSRPC")?;

    let client = Provider::<Http>::try_from(rpc)?;
    let contract = DynamicContract::new(&contract_address, &abi_json, client)?;
    let params = vec![EthAbiTokenBind::Uint {
        data: "1".to_string(),
    }];
    // [{"Uint":{"data":"1"}}]
    let json = serde_json::to_string(&params)?;
    println!("json: {}", json);
    let params2: Vec<EthAbiTokenBind> = serde_json::from_str(&json)?;
    let mycall: ContractCall<_, MyDetokenizer> = contract.function_call("ownerOf", params2)?;
    let feedback = mycall.call().await?;
    println!("mycall ok");
    println!("feedback: {:?}", feedback);
    Ok(())
}
