use crate::PrivateKey;
use anyhow::{anyhow, Result};
use defi_wallet_core_common::abi::EthAbiToken;
use defi_wallet_core_common::contract::ContractCall;
use defi_wallet_core_common::contract::DynamicContract;
use defi_wallet_core_common::EthAbiTokenBind;
use defi_wallet_core_common::EthError;
use ethers::abi::Detokenize;
use ethers::abi::InvalidOutputType;
use ethers::abi::Token;
use ethers::contract::ContractFactory;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::signers::Wallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::convert::TryFrom;
use std::sync::Arc;
type HttpProvider = Provider<Http>;
type DynamicContractHttp = DynamicContract<HttpProvider>;
type SigningWallet = Wallet<SigningKey>;
type SignerHttpWallet = SignerMiddleware<HttpProvider, SigningWallet>;
type SigningingDynamicContractHttp = DynamicContract<SignerHttpWallet>;
pub struct EthContract {
    dynamic_contract: Option<DynamicContractHttp>,
    signing_contract: Option<SigningingDynamicContractHttp>,
}

pub struct EthDetokenizer {
    json: String,
}
impl Detokenize for EthDetokenizer {
    fn from_tokens(tokens: Vec<Token>) -> std::result::Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        let json = serde_json::to_string(&tokens)
            .map_err(|e| InvalidOutputType(format!("serde json error {:?}", e,)))?;
        Ok(EthDetokenizer { json })
    }
}

#[cxx::bridge(namespace = "org::defi_wallet_core")]
#[allow(clippy::too_many_arguments)]
mod ffi {

    extern "C++" {
        include!("defi-wallet-core-cpp/src/lib.rs.h");
        type PrivateKey = crate::PrivateKey;
        type CronosTransactionReceiptRaw = crate::ffi::CronosTransactionReceiptRaw;
    }

    extern "Rust" {
        type EthContract;

        fn new_eth_contract(
            rpcserver: String,
            contact_address: String,
            abi_json: String,
        ) -> Result<Box<EthContract>>;

        fn new_signing_eth_contract(
            rpcserver: String,
            contact_address: String,
            abi_json: String,
            private_key: &PrivateKey,
        ) -> Result<Box<EthContract>>;

        // extract toplevel json with keyname
        fn extract_json(src: String, keyname: String) -> Result<String>;
        // 0x... -> bytes
        fn extract_bytes(src: String, keyname: String) -> Result<Vec<u8>>;
        // bytes -> 0x...
        fn encode_bytes(bytes: Vec<u8>) -> Result<String>;

        fn encode_deploy_contract(
            rpcserver: String,
            abi: String,
            bytecode: Vec<u8>,
            function_args: String,
        ) -> Result<Vec<u8>>;

        fn encode(
            &mut self,
            function_name: &str,
            function_args: &str, // json
        ) -> Result<Vec<u8>>;

        fn call(
            &mut self,
            function_name: &str,
            function_args: &str, // json
        ) -> Result<String>;

        fn send(
            &mut self,
            function_name: &str,
            function_args: &str, // json
        ) -> Result<CronosTransactionReceiptRaw>;
    }
} // end of ffi

fn new_eth_contract(
    rpcserver: String,
    contract_address: String,
    abi_json: String,
) -> Result<Box<EthContract>> {
    let client: Provider<Http> = Provider::<Http>::try_from(&rpcserver)?;
    let dynamic_contract: DynamicContract<Provider<Http>> =
        DynamicContract::new(&contract_address, &abi_json, client)?;
    Ok(Box::new(EthContract {
        dynamic_contract: Some(dynamic_contract),
        signing_contract: None,
    }))
}

fn new_signing_eth_contract(
    rpcserver: String,
    contract_address: String,
    abi_json: String,
    private_key: &PrivateKey,
) -> Result<Box<EthContract>> {
    let client: Provider<Http> = Provider::<Http>::try_from(&rpcserver)?;
    let signingkey: SigningKey = SigningKey::from_bytes(&private_key.to_bytes())?;
    let wallet: Wallet<SigningKey> = signingkey.into();
    let signer: SignerMiddleware<Provider<Http>, Wallet<SigningKey>> =
        SignerMiddleware::new(client, wallet);
    let signing_contract: DynamicContract<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> =
        DynamicContract::new(&contract_address, &abi_json, signer)?;
    Ok(Box::new(EthContract {
        dynamic_contract: None,
        signing_contract: Some(signing_contract),
    }))
}

fn extract_json(src: String, keyname: String) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(&src)?;
    let json = json.get(&keyname).ok_or(anyhow!("key not found"))?;
    let jsonstring = serde_json::to_string(&json)?;
    Ok(jsonstring)
}
fn extract_bytes(src: String, keyname: String) -> Result<Vec<u8>> {
    let json: serde_json::Value = serde_json::from_str(&src)?;
    let value = json.get(&keyname).ok_or(anyhow!("key not found"))?;
    let value = value
        .as_str()
        .ok_or(anyhow!("value is not string"))?
        .to_string();
    let value = if value.starts_with("0x") {
        &value[2..]
    } else {
        &value
    };
    let bytes = hex::decode(&value)?;
    Ok(bytes)
}
fn encode_bytes(bytes: Vec<u8>) -> Result<String> {
    let ret = hex::encode(&bytes);
    Ok(ret)
}

fn encode_deploy_contract(
    rpcserver: String,
    abi: String,
    bytecode: Vec<u8>,
    function_args: String,
) -> Result<Vec<u8>> {
    let params: Vec<EthAbiTokenBind> = serde_json::from_str(&function_args)?;
    let tokens = params
        .iter()
        .flat_map(EthAbiToken::try_from)
        .map(|x| Token::try_from(&x))
        .collect::<Result<Vec<Token>, _>>()?;

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

impl EthContract {
    async fn do_call(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<String> {
        let ethcontract = self
            .dynamic_contract
            .as_mut()
            .ok_or_else(|| anyhow!("contract not initialized"))?;
        let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;
        let ethcontractcall: ContractCall<_, EthDetokenizer> =
            ethcontract.function_call(function_name, params)?;
        let response: EthDetokenizer = ethcontractcall.call().await?;
        Ok(response.json)
    }

    pub fn call(&mut self, function_name: &str, function_args: &str) -> Result<String> {
        let rt = tokio::runtime::Runtime::new().map_err(|_err| EthError::AsyncRuntimeError)?;
        let res = rt.block_on(self.do_call(function_name, function_args))?;
        Ok(res)
    }

    async fn do_encode(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<Vec<u8>> {
        let ethcontract = self
            .dynamic_contract
            .as_mut()
            .ok_or_else(|| anyhow!("contract not initialized"))?;
        let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;
        let ethcontractcall: ContractCall<_, EthDetokenizer> =
            ethcontract.function_call(function_name, params)?;
        let tx: TypedTransaction = ethcontractcall.get_tx();
        let data = tx.data().ok_or_else(|| anyhow!("no data"))?;
        Ok(data.to_vec())
    }

    pub fn encode(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<Vec<u8>> {
        let rt = tokio::runtime::Runtime::new().map_err(|_err| EthError::AsyncRuntimeError)?;
        let res = rt.block_on(self.do_encode(function_name, function_args))?;
        Ok(res)
    }

    async fn do_send(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<crate::ffi::CronosTransactionReceiptRaw> {
        let ethcontract = self
            .signing_contract
            .as_mut()
            .ok_or_else(|| anyhow!("contract not initialized"))?;
        let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;
        let ethcontractcall: ContractCall<_, EthDetokenizer> =
            ethcontract.function_call(function_name, params)?;
        let ethersreceipt = ethcontractcall.send().await?;
        let defireceipt: defi_wallet_core_common::TransactionReceipt = ethersreceipt.into();
        let ret: crate::ffi::CronosTransactionReceiptRaw = defireceipt.into();
        Ok(ret)
    }
    fn send(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<crate::ffi::CronosTransactionReceiptRaw> {
        let rt = tokio::runtime::Runtime::new().map_err(|_err| EthError::AsyncRuntimeError)?;
        let res = rt.block_on(self.do_send(function_name, function_args))?;
        Ok(res)
    }
}
