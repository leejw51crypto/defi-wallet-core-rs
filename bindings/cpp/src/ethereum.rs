use anyhow::{anyhow, Result};
use defi_wallet_core_common::node::ethereum::abi::EthAbiToken;
use defi_wallet_core_common::EthAbiContract;
use defi_wallet_core_common::EthError;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::Eip1559TransactionRequest;
use std::str::FromStr;
#[derive(PartialEq)]
enum EthContractState {
    Value,
    FixedArray,
    Array,
    Tuple,
}
pub struct EthContract {
    abi_contract: EthAbiContract,
    tokens: Vec<EthAbiToken>,

    state: EthContractState,
    tmptokens: Option<Vec<EthAbiToken>>,
}

#[cxx::bridge(namespace = "org::defi_wallet_core")]
#[allow(clippy::too_many_arguments)]
mod ffi {

    extern "Rust" {
        type EthContract;

        fn new_eth_contract(abi_contract: String) -> Result<Box<EthContract>>;
        fn add_address(&mut self, address_str: &str) -> Result<()>;
        fn add_fixed_bytes(&mut self, bytes: Vec<u8>) -> Result<()>;
        fn add_bytes(&mut self, bytes: Vec<u8>) -> Result<()>;
        fn add_int(&mut self, int_str: &str) -> Result<()>;
        fn add_uint(&mut self, uint_str: &str) -> Result<()>;
        fn add_bool(&mut self, value: bool) -> Result<()>;
        fn add_string(&mut self, value: String) -> Result<()>;
        fn begin_fixed_array(&mut self) -> Result<()>;
        fn commit_fixed_array(&mut self) -> Result<()>;
        fn begin_array(&mut self) -> Result<()>;
        fn commit_array(&mut self) -> Result<()>;
        fn begin_tuple(&mut self) -> Result<()>;
        fn commit_tuple(&mut self) -> Result<()>;
        fn encode(&mut self, function_name: &str) -> Result<Vec<u8>>;
        fn decode_input(&mut self, function_name: &str, data: &[u8]) -> Result<String>;
        fn decode_output(&mut self, function_name: &str, data: &[u8]) -> Result<String>;
        fn clear_tokens(&mut self) -> Result<()>;
        fn call(
            &mut self,
            rpcserver: &str,
            contract_address: &str,
            function_name: &str,
        ) -> Result<String>;

    }
} // end of ffi

fn new_eth_contract(abi_contract: String) -> Result<Box<EthContract>> {
    let abi_contract = EthAbiContract::new(&abi_contract)?;
    let state = EthContractState::Value;
    let tmptokens = None;
    Ok(Box::new(EthContract {
        abi_contract,
        tokens: vec![],
        state,
        tmptokens,
    }))
}

impl EthContract {
    fn add_address(&mut self, address_str: &str) -> Result<()> {
        let token = EthAbiToken::from_address_str(address_str)?;
        if self.state == EthContractState::Value {
            self.tokens.push(token);
        } else if let Some(tmptokens) = &mut self.tmptokens {
            tmptokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }

        Ok(())
    }
    fn add_fixed_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let token = EthAbiToken::FixedBytes(bytes);
        if self.state == EthContractState::Value {
            self.tokens.push(token);
        } else if let Some(tmptokens) = &mut self.tmptokens {
            tmptokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }
        Ok(())
    }

    fn add_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let token = EthAbiToken::Bytes(bytes);
        if self.state == EthContractState::Value {
            self.tokens.push(token);
        } else if let Some(tmptokens) = &mut self.tmptokens {
            tmptokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }
        Ok(())
    }
    fn add_int(&mut self, int_str: &str) -> Result<()> {
        let token = EthAbiToken::from_int_str(int_str)?;
        if self.state == EthContractState::Value {
            self.tokens.push(token);
        } else if let Some(tmptokens) = &mut self.tmptokens {
            tmptokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }
        Ok(())
    }

    fn add_uint(&mut self, uint_str: &str) -> Result<()> {
        let token = EthAbiToken::from_uint_str(uint_str)?;
        if self.state == EthContractState::Value {
            self.tokens.push(token);
        } else if let Some(tmptokens) = &mut self.tmptokens {
            tmptokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }
        Ok(())
    }

    fn add_bool(&mut self, value: bool) -> Result<()> {
        let token = EthAbiToken::Bool(value);
        if self.state == EthContractState::Value {
            self.tokens.push(token);
        } else if let Some(tmptokens) = &mut self.tmptokens {
            tmptokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }
        Ok(())
    }

    fn add_string(&mut self, value: String) -> Result<()> {
        let token = EthAbiToken::String(value);
        if self.state == EthContractState::Value {
            self.tokens.push(token);
        } else if let Some(tmptokens) = &mut self.tmptokens {
            tmptokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }
        Ok(())
    }

    // fixed array
    fn begin_fixed_array(&mut self) -> Result<()> {
        if self.tmptokens.is_none() {
            self.tmptokens = Some(Vec::new());
            self.state = EthContractState::FixedArray;
        } else {
            return Err(anyhow!("tmptokens is not None"));
        }

        Ok(())
    }

    fn commit_fixed_array(&mut self) -> Result<()> {
        if self.state != EthContractState::FixedArray {
            return Err(anyhow!("state is not FixedArray"));
        }

        if let Some(tmptokens) = self.tmptokens.take() {
            let tokens = tmptokens;
            let token = EthAbiToken::FixedArray(tokens);
            self.tokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }

        Ok(())
    }

    // array
    fn begin_array(&mut self) -> Result<()> {
        if self.tmptokens.is_none() {
            self.tmptokens = Some(Vec::new());
            self.state = EthContractState::Array;
        } else {
            return Err(anyhow!("tmptokens is not None"));
        }

        Ok(())
    }
    fn commit_array(&mut self) -> Result<()> {
        if self.state != EthContractState::Array {
            return Err(anyhow!("state is not Array"));
        }

        if let Some(tmptokens) = self.tmptokens.take() {
            let tokens = tmptokens;
            let token = EthAbiToken::Array(tokens);
            self.tokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }

        Ok(())
    }

    // tuple
    fn begin_tuple(&mut self) -> Result<()> {
        if self.tmptokens.is_none() {
            self.tmptokens = Some(Vec::new());
            self.state = EthContractState::Tuple;
        } else {
            return Err(anyhow!("tmptokens is not None"));
        }
        Ok(())
    }
    fn commit_tuple(&mut self) -> Result<()> {
        if self.state != EthContractState::Tuple {
            return Err(anyhow!("state is not Tuple"));
        }
        if let Some(tmptokens) = self.tmptokens.take() {
            let tokens = tmptokens;
            let token = EthAbiToken::Tuple(tokens);
            self.tokens.push(token);
        } else {
            return Err(anyhow!("tmptokens is None"));
        }
        Ok(())
    }

    pub fn decode_input(&mut self, function_name: &str, data: &[u8]) -> Result<String> {
        let jsonoutput = self.abi_contract.decode_input(function_name, data)?;
        Ok(jsonoutput)
    }

    pub fn decode_output(&mut self, function_name: &str, data: &[u8]) -> Result<String> {
        let jsonoutput = self.abi_contract.decode_output(function_name, data)?;
        Ok(jsonoutput)
    }

    pub fn encode(&mut self, function_name: &str) -> Result<Vec<u8>> {
        let tokens = self.tokens.clone();
        let srcbytes = self.abi_contract.encode(function_name, tokens)?;
        Ok(srcbytes)
    }

    fn clear_tokens(&mut self) -> Result<()> {
        self.tokens.clear();
        self.tmptokens = None;
        self.state = EthContractState::Value;
        Ok(())
    }

    pub async fn do_call(
        &mut self,
        rpcserver: &str,
        contract_address: &str,
        function_name: &str,
    ) -> Result<String> {
        let addr = Address::from_str(contract_address)?;
        //  make ethers http provider
        let provider = Provider::<Http>::try_from(rpcserver)?;
        let data = self.tokens.clone();
        let srcbytes = self.abi_contract.encode(function_name, data)?;

        let ethbytes = Bytes::from(srcbytes);
        let tx = Eip1559TransactionRequest::new().data(ethbytes).to(addr);

        let tx = TypedTransaction::Eip1559(tx);

        let response: Bytes = provider.call(&tx, None).await?;
        let jsondata = self.decode_output(function_name, &response)?;

        Ok(jsondata)
    }
    pub fn call(
        &mut self,
        rpcserver: &str,
        contract_address: &str,
        function_name: &str,
    ) -> Result<String> {
        let rt = tokio::runtime::Runtime::new().map_err(|_err| EthError::AsyncRuntimeError)?;
        let res = rt.block_on(self.do_call(rpcserver, contract_address, function_name))?;
        Ok(res)
    }
}
