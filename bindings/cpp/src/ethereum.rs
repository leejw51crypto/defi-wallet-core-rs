use anyhow::{anyhow, Result};
use defi_wallet_core_common as common;
use defi_wallet_core_common::node::erc721::get_token_owner;
use defi_wallet_core_common::node::ethereum::abi::EthAbiToken;
use defi_wallet_core_common::{
    broadcast_contract_approval_tx, broadcast_contract_batch_transfer_tx,
    broadcast_contract_transfer_tx, broadcast_sign_eth_tx, get_contract_balance, get_eth_balance,
    ContractApproval, ContractBalance, ContractBatchTransfer, ContractTransfer, EthAbiContract,
    EthAmount, EthNetwork, WalletCoinFunc,
};
use serde::{Deserialize, Serialize};

pub struct EthContract {
    // abi_contract: EthAbiContract,
    abi_contract: String,
}

#[cxx::bridge(namespace = "org::defi_wallet_core")]
#[allow(clippy::too_many_arguments)]
mod ffi {
    #[derive(Clone)]
    pub struct MyContract {
        test1: String,
        test2: u64,
    }
    extern "Rust" {
        type EthContract;

        fn new_eth_contract() -> Result<Box<EthContract>>;
        fn test(self: &EthContract) -> Result<String>;
    }
} // end of ffi

fn new_eth_contract() -> Result<Box<EthContract>> {
    Ok(Box::new(EthContract {
        abi_contract: "".into(),
    }))
}

impl EthContract {
    fn test(&self) -> Result<String> {
        Ok("apple".into())
    }
}