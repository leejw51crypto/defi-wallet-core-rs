use crate::PrivateKey;
use anyhow::{anyhow, Result};
use defi_wallet_core_common::contract::ContractCall;
use defi_wallet_core_common::contract::DynamicContract;
use defi_wallet_core_common::EthAbiTokenBind;
use defi_wallet_core_common::EthError;
use ethers::abi::Detokenize;
use ethers::abi::InvalidOutputType;
use ethers::abi::Token;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::signers::Wallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::convert::TryFrom;

pub struct EthContract {
    abi_json: String,
    rpcserver: String,
    contract_address: String,
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
            &self,
            private_key: &PrivateKey,
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
    Ok(Box::new(EthContract {
        rpcserver,
        contract_address,
        abi_json,
    }))
}

impl EthContract {
    async fn do_call(
        &mut self,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<String> {
        let client = Provider::<Http>::try_from(&self.rpcserver)?;
        let ethcontract = DynamicContract::new(&self.contract_address, &self.abi_json, client)?;
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
        let client = Provider::<Http>::try_from(&self.rpcserver)?;
        let ethcontract = DynamicContract::new(&self.contract_address, &self.abi_json, client)?;
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
        &self,
        private_key: &PrivateKey,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<crate::ffi::CronosTransactionReceiptRaw> {
        let client = Provider::<Http>::try_from(&self.rpcserver)?;
        let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;
        let signingkey = SigningKey::from_bytes(&private_key.to_bytes())?;
        let wallet: Wallet<SigningKey> = signingkey.into();
        let signer = SignerMiddleware::new(client, wallet);

        let ethcontract = DynamicContract::new(&self.contract_address, &self.abi_json, signer)?;
        let ethcontractcall: ContractCall<_, EthDetokenizer> =
            ethcontract.function_call(function_name, params)?;

        let ethersreceipt = ethcontractcall.send().await?;
        let defireceipt: defi_wallet_core_common::TransactionReceipt = ethersreceipt.into();
        let ret: crate::ffi::CronosTransactionReceiptRaw = defireceipt.into();
        Ok(ret)
    }
    fn send(
        &self,
        private_key: &PrivateKey,
        function_name: &str,
        function_args: &str, // json
    ) -> Result<crate::ffi::CronosTransactionReceiptRaw> {
        let rt = tokio::runtime::Runtime::new().map_err(|_err| EthError::AsyncRuntimeError)?;
        let res = rt.block_on(self.do_send(private_key, function_name, function_args))?;
        Ok(res)
    }
}
