use crate::PrivateKey;
use anyhow::{anyhow, Result};
use defi_wallet_core_common::contract::ContractCall;
use defi_wallet_core_common::contract::DynamicContract;
use defi_wallet_core_common::EthAbiTokenBind;
use ethers::abi::Detokenize;
use ethers::abi::InvalidOutputType;
use ethers::abi::Token;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::signers::Wallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::convert::TryFrom;
type HttpProvider = Provider<Http>;
type DynamicContractHttp = DynamicContract<HttpProvider>;
type SigningWallet = Wallet<SigningKey>;
type SignerHttpWallet = SignerMiddleware<HttpProvider, SigningWallet>;
type SigningingDynamicContractHttp = DynamicContract<SignerHttpWallet>;
pub struct EthContract {
    dynamic_contract: Option<DynamicContractHttp>,
    signing_contract: Option<SigningingDynamicContractHttp>,
    rt: tokio::runtime::Runtime,
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
        fn read_json(filepath: String, keyname: String) -> Result<String>;

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
    let rt = tokio::runtime::Runtime::new()?;
    let client: Provider<Http> = Provider::<Http>::try_from(&rpcserver)?;
    let dynamic_contract: DynamicContract<Provider<Http>> =
        DynamicContract::new(&contract_address, &abi_json, client)?;
    Ok(Box::new(EthContract {
        dynamic_contract: Some(dynamic_contract),
        signing_contract: None,
        rt,
    }))
}

fn new_signing_eth_contract(
    rpcserver: String,
    contract_address: String,
    abi_json: String,
    private_key: &PrivateKey,
) -> Result<Box<EthContract>> {
    let rt = tokio::runtime::Runtime::new()?;
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
        rt,
    }))
}

fn read_json(filepath: String, keyname: String) -> Result<String> {
    let src = std::fs::read_to_string(filepath)?;
    let json: serde_json::Value = serde_json::from_str(&src)?;
    let json = json.get(&keyname).ok_or_else(|| anyhow!("key not found"))?;
    let jsonstring = serde_json::to_string(&json)?;
    Ok(jsonstring)
}

async fn do_send(
    ethcontract: &SigningingDynamicContractHttp,
    function_name: &str,
    function_args: &str, // json
) -> Result<crate::ffi::CronosTransactionReceiptRaw> {
    let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;
    let ethcontractcall: ContractCall<_, EthDetokenizer> =
        ethcontract.function_call(function_name, params)?;
    let ethersreceipt = ethcontractcall.send().await?;
    let defireceipt: defi_wallet_core_common::TransactionReceipt = ethersreceipt.into();
    let ret: crate::ffi::CronosTransactionReceiptRaw = defireceipt.into();
    Ok(ret)
}

async fn do_encode(
    ethcontract: &DynamicContractHttp,
    function_name: &str,
    function_args: &str, // json
) -> Result<Vec<u8>> {
    let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;
    let ethcontractcall: ContractCall<_, EthDetokenizer> =
        ethcontract.function_call(function_name, params)?;
    let tx: TypedTransaction = ethcontractcall.get_tx();
    let data = tx.data().ok_or_else(|| anyhow!("no data"))?;
    Ok(data.to_vec())
}

async fn do_call(
    ethcontract: &DynamicContractHttp,
    function_name: &str,
    function_args: &str, // json
) -> Result<String> {
    let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;
    let ethcontractcall: ContractCall<_, EthDetokenizer> =
        ethcontract.function_call(function_name, params)?;
    let response: EthDetokenizer = ethcontractcall.call().await?;
    Ok(response.json)
}

impl EthContract {
    pub fn call(&mut self, function_name: &str, function_args: &str) -> Result<String> {
        let ethcontract = self
            .dynamic_contract
            .as_mut()
            .ok_or_else(|| anyhow!("contract not initialized"))?;

        let res = self
            .rt
            .block_on(do_call(ethcontract, function_name, function_args))?;
        Ok(res)
    }

    pub fn encode(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<Vec<u8>> {
        let ethcontract = self
            .dynamic_contract
            .as_mut()
            .ok_or_else(|| anyhow!("contract not initialized"))?;

        let res = self
            .rt
            .block_on(do_encode(ethcontract, function_name, function_args))?;
        Ok(res)
    }

    fn send(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<crate::ffi::CronosTransactionReceiptRaw> {
        let ethcontract = self
            .signing_contract
            .as_mut()
            .ok_or_else(|| anyhow!("contract not initialized"))?;

        let result = self
            .rt
            .block_on(do_send(ethcontract, function_name, function_args));
        result
    }
}
