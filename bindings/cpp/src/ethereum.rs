use anyhow::{anyhow, bail, Result};
use cosmos_sdk_proto::ibc::applications::interchain_accounts::v1::Type;
use defi_wallet_core_common::contract::ContractCall;
use defi_wallet_core_common::contract::DynamicContract;
use defi_wallet_core_common::node::ethereum::abi::EthAbiToken;
use defi_wallet_core_common::EthAbiContract;
use defi_wallet_core_common::EthAbiTokenBind;
use defi_wallet_core_common::EthError;
use ethers::abi::InvalidOutputType;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::Eip1559TransactionRequest;
use std::convert::TryFrom;
use std::str::FromStr;
// use Detokenizer
use ethers::abi::Detokenize;
use ethers::abi::Token;
#[derive(PartialEq, Debug)]
enum EthContractState {
    Value,
    FixedArray,
    Array,
    Tuple,
}

pub struct EthAbiTokenWrapper {
    token: EthAbiToken,
}
pub struct EthContract {
    abi_contract: EthAbiContract,
    abi_json: String,
    tokens: Vec<EthAbiToken>,

    state: EthContractState,
    tmptokens: Option<Vec<EthAbiToken>>,
}

pub struct MyDetokenizer {
    json: String,
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

#[cxx::bridge(namespace = "org::defi_wallet_core")]
#[allow(clippy::too_many_arguments)]
mod ffi {
    extern "Rust" {
        type EthAbiTokenWrapper;

    }

    extern "Rust" {
        type EthContract;

        fn new_eth_contract(abi_json: String) -> Result<Box<EthContract>>;

        fn encode(
            &mut self,
            rpcserver: &str,
            contract_address: &str,
            function_name: &str,
            function_args: &str, // json
        ) -> Result<Vec<u8>>;

        fn send(
            &mut self,
            rpcserver: &str,
            contract_address: &str,
            function_name: &str,
            function_args: &str, // json
        ) -> Result<String>;

        fn call(
            &mut self,
            rpcserver: &str,
            contract_address: &str,
            function_name: &str,
            function_args: &str, // json
        ) -> Result<String>;

    }
} // end of ffi

fn new_eth_contract(abi_json: String) -> Result<Box<EthContract>> {
    let abi_contract = EthAbiContract::new(&abi_json)?;
    let state = EthContractState::Value;
    let tmptokens = None;
    Ok(Box::new(EthContract {
        abi_contract,
        abi_json,
        tokens: vec![],
        state,
        tmptokens,
    }))
}

impl EthContract {
    async fn do_call(
        &mut self,
        rpcserver: &str,
        contract_address: &str,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<String> {
        let client = Provider::<Http>::try_from(rpcserver)?;

        println!("contract_address: {}", contract_address);
        println!("function_name: {}", function_name);
        let mycontract = DynamicContract::new(contract_address, &self.abi_json, client)?;

        let params: Vec<EthAbiTokenBind> = serde_json::from_str(&function_args)?;

        let mycontractcall: ContractCall<_, MyDetokenizer> =
            mycontract.function_call(function_name, params)?;

        let response: MyDetokenizer = mycontractcall.call().await?;

        Ok(response.json.into())
    }

    pub fn call(
        &mut self,
        rpcserver: &str,
        contract_address: &str,
        function_name: &str,
        function_args: &str,
    ) -> Result<String> {
        let rt = tokio::runtime::Runtime::new().map_err(|_err| EthError::AsyncRuntimeError)?;
        let res =
            rt.block_on(self.do_call(rpcserver, contract_address, function_name, function_args))?;
        Ok(res)
    }

    async fn do_encode(
        &mut self,
        rpcserver: &str,
        contract_address: &str,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<Vec<u8>> {
        let client = Provider::<Http>::try_from(rpcserver)?;

        let mycontract = DynamicContract::new(contract_address, &self.abi_json, client)?;
        
        let params: Vec<EthAbiTokenBind> = serde_json::from_str(&function_args)?;
        println!("do_encode params: {:?}", params);
        let mycontractcall: ContractCall<_, MyDetokenizer> =
            mycontract.function_call(function_name, params)?;

        let tx:TypedTransaction=mycontractcall.get_tx();
        let data=tx.data().ok_or_else(|| anyhow!("no data"))?;

        Ok(data.to_vec())
    }

    pub fn encode(
        &mut self,
        rpcserver: &str,
        contract_address: &str,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<Vec<u8>> {
        let rt = tokio::runtime::Runtime::new().map_err(|_err| EthError::AsyncRuntimeError)?;
        let res =
            rt.block_on(self.do_encode(rpcserver, contract_address, function_name, function_args))?;
        Ok(res)
    }

    pub fn send(
        &mut self,
        rpcserver: &str,
        contract_address: &str,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<String> //json
    {
        Ok("".to_string())
    }
}
