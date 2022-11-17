use std::sync::Arc;

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

#[derive(PartialEq)]
enum EthContractState {
    VALUE,
    FIXED_ARRAY,
    ARRAY,
    TUPLE,
}
pub struct EthContract {
    abi_contract: EthAbiContract,
    tokens: Vec<EthAbiToken>,

    state: EthContractState,
    tmptokens: Vec<EthAbiToken>,
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

        fn new_eth_contract(abi_contract: String) -> Result<Box<EthContract>>;
        fn test(self: &EthContract) -> Result<String>;
    }
} // end of ffi

fn new_eth_contract(abi_contract: String) -> Result<Box<EthContract>> {
    let abi_contract = EthAbiContract::new(&abi_contract)?;
    let state=EthContractState::VALUE;
    let tmptokens=Vec::new();
    Ok(Box::new(EthContract { abi_contract , tokens: vec![],state , tmptokens}))
}

impl EthContract {
    fn test(&self) -> Result<String> {
        Ok("apple".into())
    }
    fn add_address(&mut self, address_str: &str) -> Result<()> {
        let token = EthAbiToken::from_address_str(address_str)?;
        if self.state == EthContractState::VALUE{
            self.tokens.push(token);
        }
        else{
            self.tmptokens.push(token);
        }

        Ok(())
    }
    fn add_fixed_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let token=EthAbiToken::FixedBytes(bytes);
        if self.state == EthContractState::VALUE{
            self.tokens.push(token);
        }
        else{
            self.tmptokens.push(token);
        }
        Ok(())
    }

    fn add_bytes(&mut self,bytes: Vec<u8>) -> Result<()> {
        let token=EthAbiToken::Bytes(bytes);
        if self.state == EthContractState::VALUE{
            self.tokens.push(token);
        }
        else{
            self.tmptokens.push(token);
        }
        Ok(())
    }
    fn add_int(&mut self,int_str: &str) -> Result<()> {
        let token = EthAbiToken::from_int_str(int_str)?;
        if self.state == EthContractState::VALUE{
            self.tokens.push(token);
        }
        else{
            self.tmptokens.push(token);
        }
        Ok(())
    }

    fn add_uint(&mut self,uint_str: &str) -> Result<()> {
        let token = EthAbiToken::from_uint_str(uint_str)?;
        if self.state == EthContractState::VALUE{
            self.tokens.push(token);
        }
        else{
            self.tmptokens.push(token);
        }
        Ok(())
    }

    fn add_bool(&mut self,value: bool) -> Result<()> {
        let token = EthAbiToken::Bool(value);
        if self.state == EthContractState::VALUE{
            self.tokens.push(token);
        }
        else{
            self.tmptokens.push(token);
        }
        Ok(())
    }

    fn add_string(&mut self,value: String) -> Result<()> {
        let token=EthAbiToken::String(value);
        if self.state == EthContractState::VALUE{
            self.tokens.push(token);
        }
        else{
            self.tmptokens.push(token);
        }
        Ok(())
    }

    // fixed array
    fn begin_fixed_array(&mut self) -> Result<()> {
        self.tmptokens.clear();
        self.state=EthContractState::FIXED_ARRAY;
        Ok(())
    }

    fn commit_fixed_array(&mut self) -> Result<()> {
        // move self.tokens
        let mut tokens=Vec::new();
        std::mem::swap(&mut tokens,&mut self.tokens);

        let token=EthAbiToken::FixedArray(tokens);
        self.tmptokens.clear();
        self.tokens.push(token);
        Ok(())
    }

    // array
    fn begin_array(&mut self) -> Result<()> {
        self.tmptokens.clear();
        self.state=EthContractState::ARRAY;
        Ok(())
    }
    fn commit_array(&mut self) -> Result<()> {
        let mut tokens=Vec::new();
        std::mem::swap(&mut tokens,&mut self.tokens);

        let token=EthAbiToken::Array(tokens);
        self.tmptokens.clear();
        self.tokens.push(token);
        Ok(())
    }

    // tuple
    fn begin_tuple(&mut self) -> Result<()> {
        self.tmptokens.clear();
        self.state=EthContractState::TUPLE;
        Ok(())
    }
    fn commit_tuple(&mut self) -> Result<()> {
        let mut tokens=Vec::new();
        std::mem::swap(&mut tokens,&mut self.tokens);

        let token=EthAbiToken::Tuple(tokens);
        self.tmptokens.clear();
        self.tokens.push(token);
        Ok(())
    }
}
