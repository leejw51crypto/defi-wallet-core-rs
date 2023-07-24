use anyhow::Result;
use defi_wallet_core_common::abi::EthAbiToken;
use defi_wallet_core_common::EthAbiTokenBind;
use defi_wallet_core_common::EthError;
use ethers::abi::Token;
fn main() -> Result<()> {
    println!("speak");
    let function_args = r#"[
        {
            "Uint": {
                "data": "100"
            }
        },
        {
            "Array": {
                "data": [
                    {
                        "Address": {
                            "data": "0x1234567890ABCDEFabcdefABCDEF1234567890ab"
                        }
                    },
                    {
                        "Address": {
                            "data": "0xABCDEFabcdef1234567890ABCDEFabcdef123456"
                        }
                    },
                    {
                        "Address": {
                            "data": "0xabcdef1234567890ABCDEFabcdef1234567890AB"
                        }
                    }
                ]
            }
        },
        {
            "Address": {
                "data": "0x9876543210ABCDEFabcdefABCDEF9876543210ab"
            }
        },
        {
            "Uint": {
                "data": "200"
            }
        }
    ]"#;
    let params: Vec<EthAbiTokenBind> = serde_json::from_str(function_args)?;

    let tokens = params
        .iter()
        .flat_map(EthAbiToken::try_from)
        .map(|x| Token::try_from(&x))
        .collect::<Result<Vec<Token>, _>>()?;

    println!("params: {:?}", params);
    println!("{}", serde_json::to_string(&params)?);
    println!("tokens: {:?}", tokens);
    Ok(())
}
