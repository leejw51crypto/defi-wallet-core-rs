use crate::{EthNetwork, Network};
use bip39::{Language, Mnemonic};
use cosmrs::bip32::secp256k1::ecdsa::SigningKey;
use cosmrs::bip32::{self, DerivationPath, PrivateKey, Seed, XPrv};
use cosmrs::crypto::PublicKey;
use ethers::core::k256::ecdsa;
use ethers::prelude::{LocalWallet, Signature, Signer, H256};
use ethers::utils::hex::{self, FromHexError, ToHex};
use ethers::utils::{hash_message, secret_key_to_address};
use rand_core::{OsRng, RngCore};
use secrecy::{ExposeSecret, SecretString, Zeroize};
use std::str::FromStr;
use std::sync::Arc;

/// wasm binding related functions
mod wasm_binding;

#[cfg(target_arch = "wasm32")]
pub use wasm_binding::*;

/// describes what coin type to use (for HD derivation or address generation)
#[derive(Clone)]
pub enum WalletCoin {
    CosmosSDK { network: Network },
    Ethereum { network: EthNetwork },
}

/// describes the number of words in mnemonic
pub enum MnemonicWordCount {
    /// Word 12
    Twelve,
    /// Word 18
    Eighteen,
    /// Word 24
    TwentyFour,
}

impl From<MnemonicWordCount> for usize {
    fn from(word_count: MnemonicWordCount) -> usize {
        match word_count {
            MnemonicWordCount::Twelve => 12,
            MnemonicWordCount::Eighteen => 18,
            MnemonicWordCount::TwentyFour => 24,
        }
    }
}

pub struct WalletCoinFunc {
    pub coin: WalletCoin,
}

impl WalletCoinFunc {
    pub fn new(coin: WalletCoin) -> Self {
        Self { coin }
    }

    /// get address from a private key
    pub fn derive_address(&self, private_key: &SecretKey) -> Result<String, HdWrapError> {
        match &self.coin {
            WalletCoin::CosmosSDK { network } => {
                let bech32_hrp = network.get_bech32_hrp();
                let pubkey = PublicKey::from(private_key.get_signing_key().public_key());
                pubkey
                    .account_id(bech32_hrp)
                    .map(|x| x.to_string())
                    .map_err(HdWrapError::AccountId)
            }
            WalletCoin::Ethereum { .. } => {
                let address = secret_key_to_address(&private_key.get_signing_key());
                let address_hex: String = address.encode_hex();
                Ok(format!("0x{}", address_hex))
            }
        }
    }

    /// return the HD coin type
    pub fn get_coin_type(&self) -> u32 {
        match &self.coin {
            WalletCoin::CosmosSDK { network } => network.get_coin_type(),
            WalletCoin::Ethereum { .. } => 60,
        }
    }

    /// return ethereum like chain network
    pub fn get_eth_network(&self) -> EthNetwork {
        match &self.coin {
            WalletCoin::Ethereum { network } => network.clone(),
            _ => Default::default(),
        }
    }
}

/// BIP32-style wallet that can be backed up to and recovered from BIP39
pub struct HDWallet {
    seed: Seed,
    mnemonic: Option<Mnemonic>,
}

/// wrapper around HD Wallet errors
#[derive(Debug, thiserror::Error)]
pub enum HdWrapError {
    #[error("The length should be 64-bytes")]
    InvalidLength,
    #[error("HD wallet error (bip 39): {0}")]
    HDErrorBip39(bip39::Error),
    #[error("HD wallet error (bip 32): {0}")]
    HDErrorBip32(bip32::Error),
    #[error("AccountId error: {0}")]
    AccountId(eyre::Report),
}

impl HDWallet {
    /// constructs a new HD wallet from the seed value
    /// returns an error if the seed doesn't have a correct length
    pub fn new(mut seed_val: Vec<u8>) -> Result<Self, HdWrapError> {
        const SEED_LEN: usize = 64;
        if seed_val.len() != SEED_LEN {
            Err(HdWrapError::InvalidLength)
        } else {
            let mut seed = [0u8; SEED_LEN];
            seed.copy_from_slice(&seed_val);
            seed_val.zeroize();
            Ok(HDWallet {
                seed: Seed::new(seed),
                mnemonic: None,
            })
        }
    }

    /// generates the HD wallet with a BIP39 backup phrase (English words)
    pub fn generate_wallet(
        password: Option<String>,
        word_count: Option<MnemonicWordCount>,
    ) -> Result<Self, HdWrapError> {
        let pass = SecretString::new(password.unwrap_or_default());
        let word_count = word_count.unwrap_or(MnemonicWordCount::TwentyFour);
        HDWallet::generate_english(pass, word_count)
    }

    /// build new HD wallet with a BIP39 backup phrase (English words) and password
    /// used in extension
    pub fn new_wallet(
        password: Option<String>,
        word_count: Option<MnemonicWordCount>,
    ) -> Result<Self, HdWrapError> {
        let pass = SecretString::new(password.unwrap_or_default());
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);
        let size: usize = word_count.unwrap_or(MnemonicWordCount::TwentyFour).into();
        let entropy_bytes = (size / 3) * 4;
        let phrase = Mnemonic::from_entropy_in(Language::English, &entropy[0..entropy_bytes])
            .map_err(HdWrapError::HDErrorBip39)?;
        Self::recover_english(SecretString::new(phrase.to_string()), pass)
    }

    /// recovers/imports HD wallet from a BIP39 backup phrase (English words) and password
    pub fn recover_wallet(
        mnemonic_phrase: String,
        password: Option<String>,
    ) -> Result<Self, HdWrapError> {
        let phrase = SecretString::new(mnemonic_phrase);
        let pass = SecretString::new(password.unwrap_or_default());
        Self::recover_english(phrase, pass)
    }

    /// returns the backup mnemonic phrase (if any)
    pub fn get_backup_mnemonic_phrase(&self) -> Option<String> {
        self.mnemonic.as_ref().map(|m| m.to_string())
    }

    /// generates the HD wallet and returns the backup phrase
    fn generate_english(
        password: SecretString,
        word_count: MnemonicWordCount,
    ) -> Result<Self, HdWrapError> {
        let mut rng = OsRng;
        let word_count_usize: usize = word_count.into();
        let entropy_bytes = (word_count_usize / 3) * 4;
        const MAX_NB_WORDS: usize = 24;
        let mut entropy = [0u8; (MAX_NB_WORDS / 3) * 4];
        rand_core::RngCore::fill_bytes(&mut rng, &mut entropy[0..entropy_bytes]);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy[0..entropy_bytes])
            .map_err(HdWrapError::HDErrorBip39)?;
        let seed = mnemonic.to_seed_normalized(password.expose_secret());
        let seed = Seed::new(seed);
        Ok(Self {
            seed,
            mnemonic: Some(mnemonic),
        })
    }

    /// recovers the HD wallet from a backup phrase
    fn recover_english(
        mnemonic_phrase: SecretString,
        password: SecretString,
    ) -> Result<Self, HdWrapError> {
        let mnemonic = Mnemonic::from_str(mnemonic_phrase.expose_secret())
            .map_err(HdWrapError::HDErrorBip39)?;
        let seed = mnemonic.to_seed_normalized(password.expose_secret());
        let seed = Seed::new(seed);

        Ok(Self {
            seed,
            mnemonic: Some(mnemonic),
        })
    }

    /// returns the address from index in wallet
    pub fn get_address(&self, coin: WalletCoin, index: u32) -> Result<String, HdWrapError> {
        let pkey = self.get_key_from_index(coin.clone(), index)?;
        pkey.to_address(coin)
    }

    /// returns the default address of the wallet
    pub fn get_default_address(&self, coin: WalletCoin) -> Result<String, HdWrapError> {
        self.get_address(coin, 0)
    }

    /// return the secret key for a given derivation path
    pub fn get_key(&self, derivation_path: String) -> Result<Arc<SecretKey>, HdWrapError> {
        let derivation_path: DerivationPath =
            derivation_path.parse().map_err(HdWrapError::HDErrorBip32)?;
        let child_xprv = XPrv::derive_from_path(&self.seed, &derivation_path)
            .map_err(HdWrapError::HDErrorBip32)?;
        Ok(Arc::new(SecretKey(child_xprv.private_key().clone())))
    }

    /// return the secret key for a given coin and index
    pub fn get_key_from_index(
        &self,
        coin: WalletCoin,
        index: u32,
    ) -> Result<Arc<SecretKey>, HdWrapError> {
        let coin_type = WalletCoinFunc { coin }.get_coin_type();
        let derivation_path: DerivationPath = format!("m/44'/{}'/0'/0/{}", coin_type, index)
            .parse()
            .map_err(HdWrapError::HDErrorBip32)?;
        let child_xprv = XPrv::derive_from_path(&self.seed, &derivation_path)
            .map_err(HdWrapError::HDErrorBip32)?;
        Ok(Arc::new(SecretKey(child_xprv.private_key().clone())))
    }
}

/// wrapper around Secret Key errors
#[derive(Debug, thiserror::Error)]
pub enum SecretKeyWrapError {
    #[error("Invalid bytes: {0}")]
    InvalidBytes(ecdsa::Error),
    #[error("Invalid hex: {0}")]
    InvalidHex(FromHexError),
}

/// wrapper around secp256k1 signing key
pub struct SecretKey(SigningKey);

impl SecretKey {
    /// generates a random secret key
    pub fn new() -> Self {
        Self(SigningKey::random(&mut OsRng))
    }

    /// constructs secret key from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SecretKeyWrapError> {
        let signing_key =
            SigningKey::from_bytes(bytes.as_slice().into()).map_err(SecretKeyWrapError::InvalidBytes)?;
        Ok(Self(signing_key))
    }

    /// constructs secret key from hex
    pub fn from_hex(hex: String) -> Result<Self, SecretKeyWrapError> {
        let bytes = hex::decode(hex).map_err(SecretKeyWrapError::InvalidHex)?;
        Self::from_bytes(bytes)
    }

    /// gets the inner signing key
    pub fn get_signing_key(&self) -> SigningKey {
        self.0.clone()
    }

    /// signs an arbitrary message as per EIP-191
    /// TODO: chain_id may not be necessary?
    pub fn eth_sign(&self, message: &[u8], chain_id: u64) -> Result<Signature, HdWrapError> {
        let hash = hash_message(message);
        let wallet = LocalWallet::from(self.get_signing_key()).with_chain_id(chain_id);
        let signature = wallet.sign_hash(hash).unwrap();
        Ok(signature)
    }

    // eth sign hash hex string without the 0x prefix
    pub fn eth_sign_by_hash(&self, hash: String, chain_id: u64) -> Result<Signature, HdWrapError> {
        let vhash = hex_to_bytes(hash);
        let bhash: [u8; 32] = vhash.try_into().unwrap();
        let uhash: H256 = bhash.into();
        let wallet = LocalWallet::from(self.get_signing_key()).with_chain_id(chain_id);
        let signature = wallet.sign_hash(uhash).unwrap();
        Ok(signature)
    }

    /// gets public key to byte array
    pub fn get_public_key_bytes(&self) -> Vec<u8> {
        self.0.clone().public_key().to_sec1_bytes().to_vec()
    }

    /// gets public key to a hex string without the 0x prefix
    pub fn get_public_key_hex(&self) -> String {
        hex::encode(self.0.clone().public_key().to_sec1_bytes())
    }

    /// converts private key to byte array
    pub fn to_bytes(&self) -> Vec<u8> {
        self.get_signing_key().to_bytes().to_vec()
    }

    /// converts private key to a hex string without the 0x prefix
    pub fn to_hex(&self) -> String {
        hex::encode(self.get_signing_key().to_bytes())
    }

    /// converts private to address with coin type
    pub fn to_address(&self, coin: WalletCoin) -> Result<String, HdWrapError> {
        WalletCoinFunc { coin }
            .derive_address(self)
            .map_err(|e| HdWrapError::AccountId(e.into()))
    }
}

impl Default for SecretKey {
    fn default() -> Self {
        Self::new()
    }
}

impl From<SigningKey> for SecretKey {
    fn from(signing_key: SigningKey) -> Self {
        Self(signing_key)
    }
}

/// Convert byte array to a hex string without the 0x prefix
pub fn bytes_to_hex(data: Vec<u8>) -> String {
    hex::encode(data)
}

/// Convert hex string to byte array, hex string without the 0x prefix
pub fn hex_to_bytes(data: String) -> Vec<u8> {
    hex::decode(data).expect("hex data error")
}

#[cfg(test)]
mod hd_wallet_tests {
    use super::*;

    #[test]
    fn test_generate_24_word_mnemonic_wallet_as_default() {
        let wallet = HDWallet::generate_wallet(None, None).expect("Failed to generate wallet");
        let mnemonic_phrase = wallet
            .get_backup_mnemonic_phrase()
            .expect("Failed to get backup mnemonic phrase");
        let words = mnemonic_phrase.split(' ');
        assert_eq!(words.count(), 24);
    }

    #[test]
    fn test_generate_wallet_for_12_word_mnemonic() {
        let wallet = HDWallet::generate_wallet(None, Some(MnemonicWordCount::Twelve))
            .expect("Failed to generate wallet");
        let mnemonic_phrase = wallet
            .get_backup_mnemonic_phrase()
            .expect("Failed to get backup mnemonic phrase");
        let words = mnemonic_phrase.split(' ');
        assert_eq!(words.count(), 12);
    }

    #[test]
    fn test_generate_wallet_for_18_word_mnemonic() {
        let wallet = HDWallet::generate_wallet(None, Some(MnemonicWordCount::Eighteen))
            .expect("Failed to generate wallet");
        let mnemonic_phrase = wallet
            .get_backup_mnemonic_phrase()
            .expect("Failed to get backup mnemonic phrase");
        let words = mnemonic_phrase.split(' ');
        assert_eq!(words.count(), 18);
    }

    #[test]
    fn test_generate_wallet_for_24_word_mnemonic() {
        let wallet = HDWallet::generate_wallet(None, Some(MnemonicWordCount::TwentyFour))
            .expect("Failed to generate wallet");
        let mnemonic_phrase = wallet
            .get_backup_mnemonic_phrase()
            .expect("Failed to get backup mnemonic phrase");
        let words = mnemonic_phrase.split(' ');
        assert_eq!(words.count(), 24);
    }

    #[test]
    fn test_wallet_recovered_from_12_word_mnemonic() {
        let words = "guard input oyster oyster slot doctor repair shed soon assist blame power";

        let wallet = HDWallet::recover_wallet(words.to_owned(), Some("".to_owned()))
            .expect("Failed to recover wallet");
        assert_eq!(wallet.get_backup_mnemonic_phrase(), Some(words.to_owned()));

        let default_cosmos_address = wallet
            .get_default_address(WalletCoin::CosmosSDK {
                network: Network::CryptoOrgMainnet,
            })
            .expect("Failed to get default Cosmos address");
        assert_eq!(
            default_cosmos_address,
            "cro16edxe89pn8ly9c7cy702x9e62fdvf3k9tnzycj"
        );

        let default_eth_address = wallet
            .get_default_address(WalletCoin::Ethereum {
                network: EthNetwork::Mainnet,
            })
            .expect("Failed to get default Eth address");
        assert_eq!(
            default_eth_address,
            "0xda25e7a4b1bda34e303e6d7f22abef78ce9a55db"
        );

        let cosmos_address = wallet
            .get_address(
                WalletCoin::CosmosSDK {
                    network: Network::CryptoOrgMainnet,
                },
                1,
            )
            .expect("Failed to get Cosmos address");
        assert_eq!(cosmos_address, "cro1keycl6d55fnlzwgfdufl53vuf95uvxnry6uj2q");

        let eth_address = wallet
            .get_address(
                WalletCoin::Ethereum {
                    network: EthNetwork::Mainnet,
                },
                1,
            )
            .expect("Failed to get Eth address");
        assert_eq!(eth_address, "0x74aeb73c4f6c10750bcd8608b0347f3e4750151c");

        let private_key = wallet
            .get_key("m/44'/394'/0'/0/0".to_string())
            .expect("key");
        let raw_key = private_key.0.to_bytes().to_vec();
        let expected_key = [
            46, 156, 107, 197, 216, 223, 81, 119, 105, 126, 144, 232, 123, 208, 152, 210, 214, 22,
            95, 9, 97, 149, 215, 143, 118, 204, 161, 206, 203, 243, 117, 37,
        ]
        .to_vec();
        assert_eq!(raw_key, expected_key);
    }

    #[test]
    fn test_wallet_recovered_from_18_word_mnemonic() {
        let words = "kingdom donate chunk chapter hotel cigar diagram steel sunny grab allow ranch witness reveal window grunt slogan damp";

        let wallet = HDWallet::recover_wallet(words.to_owned(), Some("".to_owned()))
            .expect("Failed to recover wallet");
        assert_eq!(wallet.get_backup_mnemonic_phrase(), Some(words.to_owned()));

        let default_cosmos_address = wallet
            .get_default_address(WalletCoin::CosmosSDK {
                network: Network::CryptoOrgMainnet,
            })
            .expect("Failed to get default Cosmos address");
        assert_eq!(
            default_cosmos_address,
            "cro1cvqgv7qaxdv9j9yswttr8xndyyyf30wfczx936"
        );

        let default_eth_address = wallet
            .get_default_address(WalletCoin::Ethereum {
                network: EthNetwork::Mainnet,
            })
            .expect("Failed to get default Eth address");
        assert_eq!(
            default_eth_address,
            "0xa585a184592f9dd0a9d003a894aac7175fbbfc2d"
        );

        let cosmos_address = wallet
            .get_address(
                WalletCoin::CosmosSDK {
                    network: Network::CryptoOrgMainnet,
                },
                1,
            )
            .expect("Failed to get Cosmos address");
        assert_eq!(cosmos_address, "cro1nx9ctly98zzu98ucvxmgzf0km7aqll8mlx4636");

        let eth_address = wallet
            .get_address(
                WalletCoin::Ethereum {
                    network: EthNetwork::Mainnet,
                },
                1,
            )
            .expect("Failed to get Eth address");
        assert_eq!(eth_address, "0x2d78f7508a87167b7e3f4ef3d4eed57015ef7f9f");

        let private_key = wallet
            .get_key("m/44'/394'/0'/0/0".to_string())
            .expect("key");
        let raw_key = private_key.0.to_bytes().to_vec();
        let expected_key = [
            109, 109, 61, 65, 229, 60, 215, 185, 187, 147, 87, 20, 111, 211, 39, 93, 111, 191, 18,
            182, 56, 57, 234, 255, 85, 97, 144, 12, 42, 244, 105, 38,
        ]
        .to_vec();
        assert_eq!(raw_key, expected_key);
    }

    #[test]
    fn test_wallet_recovered_from_24_word_mnemonic() {
        let words = "dune car envelope chuckle elbow slight proud fury remove candy uphold puzzle call select sibling sport gadget please want vault glance verb damage gown";

        let wallet = HDWallet::recover_wallet(words.to_owned(), Some("".to_owned()))
            .expect("Failed to recover wallet");
        assert_eq!(wallet.get_backup_mnemonic_phrase(), Some(words.to_owned()));

        let default_cosmos_address = wallet
            .get_default_address(WalletCoin::CosmosSDK {
                network: Network::CryptoOrgMainnet,
            })
            .expect("Failed to get default Cosmos address");
        assert_eq!(
            default_cosmos_address,
            "cro1u9q8mfpzhyv2s43js7l5qseapx5kt3g2rf7ppf"
        );

        let default_eth_address = wallet
            .get_default_address(WalletCoin::Ethereum {
                network: EthNetwork::Mainnet,
            })
            .expect("Failed to get default Eth address");
        assert_eq!(
            default_eth_address,
            "0x2c600e0a72b3ae39e9b27d2e310b180abe779368"
        );

        let cosmos_address = wallet
            .get_address(
                WalletCoin::CosmosSDK {
                    network: Network::CryptoOrgMainnet,
                },
                1,
            )
            .expect("Failed to get Cosmos address");
        assert_eq!(cosmos_address, "cro1g8w7w0kdx0hfv4eqhmv8avxnf7qruchg9pk3v2");

        let eth_address = wallet
            .get_address(
                WalletCoin::Ethereum {
                    network: EthNetwork::Mainnet,
                },
                1,
            )
            .expect("Failed to get Eth address");
        assert_eq!(eth_address, "0x5a64bef6db23fc854e79eea9e630ccb9301629cb");

        let private_key = wallet
            .get_key("m/44'/394'/0'/0/0".to_string())
            .expect("key");
        let raw_key = private_key.0.to_bytes().to_vec();
        let expected_key = [
            212, 154, 121, 125, 182, 59, 97, 193, 72, 209, 118, 126, 97, 111, 241, 92, 61, 217,
            200, 59, 99, 203, 166, 28, 33, 142, 161, 114, 242, 56, 98, 42,
        ]
        .to_vec();
        assert_eq!(raw_key, expected_key);
    }

    #[test]
    fn test_get_key_from_index() {
        let words = "lumber flower voice hood obvious behave relax chief warm they they mountain";

        let wallet = HDWallet::recover_wallet(words.to_owned(), Some("".to_owned()))
            .expect("Failed to recover wallet");
        let key = wallet
            .get_key_from_index(
                WalletCoin::Ethereum {
                    network: EthNetwork::BSC,
                },
                1,
            )
            .expect("get_key_from_index error");
        assert_eq!(
            key.to_hex(),
            "6f53576748877b603718b1aa1e7106aec5e15c1a2f39ea8c4683ac0d5a435a13"
        );
        let address = key
            .to_address(WalletCoin::Ethereum {
                network: EthNetwork::BSC,
            })
            .expect("address error");
        assert_eq!(address, "0x68418d0fdb846e8736aa613159035a9d9fde11f0");
    }
}

#[cfg(test)]
mod secret_key_tests {
    use super::*;

    #[test]
    fn test_generate_random_secret_key() {
        let secret_key = SecretKey::new();
        secret_key.get_signing_key();
        secret_key
            .eth_sign("hello world".as_bytes(), 1)
            .expect("failed to sign a message");

        assert!(!secret_key.get_public_key_bytes().is_empty());
        assert!(!secret_key.get_public_key_hex().is_empty());
        assert!(!secret_key.to_bytes().is_empty());
        assert!(!secret_key.to_hex().is_empty());
    }

    #[test]
    fn test_construct_secret_key_from_bytes() {
        let bytes = [
            34, 132, 105, 223, 8, 11, 89, 187, 229, 227, 66, 38, 131, 228, 149, 134, 208, 32, 112,
            118, 177, 151, 63, 38, 193, 73, 194, 226, 198, 187, 100, 133,
        ];

        let secret_key = SecretKey::from_bytes(bytes.to_vec())
            .expect("Failed to construct Secret Key from bytes");
        secret_key.get_signing_key();
        secret_key
            .eth_sign("hello world".as_bytes(), 1)
            .expect("failed to sign a message");

        assert_eq!(
            secret_key.get_public_key_bytes(),
            [
                2, 31, 14, 65, 53, 132, 187, 5, 189, 214, 210, 70, 194, 21, 71, 128, 144, 69, 201,
                166, 84, 68, 242, 68, 100, 68, 215, 215, 113, 29, 5, 15, 97
            ]
        );
        assert_eq!(
            secret_key.get_public_key_hex(),
            "021f0e413584bb05bdd6d246c21547809045c9a65444f2446444d7d7711d050f61"
        );
        assert_eq!(secret_key.to_bytes(), bytes);
        assert_eq!(
            secret_key.to_hex(),
            "228469df080b59bbe5e3422683e49586d0207076b1973f26c149c2e2c6bb6485"
        );
    }

    #[test]
    fn test_construct_secret_key_from_hex() {
        let hex = "e7de4e2f72573cf3c6e1fa3845cec6a4e2aac582702cac14bb9da0bb05aa24ae";

        let secret_key =
            SecretKey::from_hex(hex.to_owned()).expect("Failed to construct Secret Key from hex");
        secret_key.get_signing_key();
        secret_key
            .eth_sign("hello world".as_bytes(), 1)
            .expect("failed to sign a message");

        assert_eq!(
            secret_key.get_public_key_bytes(),
            [
                3, 206, 250, 179, 248, 156, 98, 236, 197, 76, 9, 99, 69, 22, 187, 40, 25, 210, 13,
                131, 117, 121, 86, 199, 244, 105, 13, 195, 184, 6, 236, 199, 210
            ]
        );
        assert_eq!(
            secret_key.get_public_key_hex(),
            "03cefab3f89c62ecc54c09634516bb2819d20d83757956c7f4690dc3b806ecc7d2"
        );
        assert_eq!(
            secret_key.to_bytes(),
            [
                231, 222, 78, 47, 114, 87, 60, 243, 198, 225, 250, 56, 69, 206, 198, 164, 226, 170,
                197, 130, 112, 44, 172, 20, 187, 157, 160, 187, 5, 170, 36, 174
            ]
        );
        assert_eq!(secret_key.to_hex(), hex);
    }

    #[test]
    fn test_secret_key_address() {
        let hex = "24e585759e492f5e810607c82c202476c22c5876b10247ebf8b2bb7f75dbed2e";
        let secret_key =
            SecretKey::from_hex(hex.to_owned()).expect("Failed to construct Secret Key from hex");

        assert_eq!(
            secret_key.get_public_key_hex(),
            "02059b1fc4b7834d77765a024b6c52f570f19ed5113d8cedea0b90fbae39edda1c"
        );
        let address = secret_key
            .to_address(WalletCoin::Ethereum {
                network: EthNetwork::Mainnet,
            })
            .expect("get key address error");
        assert_eq!(address, "0x714e0ed767d99f8be2b789f9dd1e2113de8eac53");
    }

    #[test]
    fn test_eth_sign() {
        let words = "lumber flower voice hood obvious behave relax chief warm they they mountain";

        let wallet = HDWallet::recover_wallet(words.to_owned(), Some("".to_owned()))
            .expect("Failed to recover wallet");
        let key = wallet
            .get_key_from_index(
                WalletCoin::Ethereum {
                    network: EthNetwork::BSC,
                },
                0,
            )
            .expect("get_key_from_index error");

        let address = key
            .to_address(WalletCoin::Ethereum {
                network: EthNetwork::BSC,
            })
            .expect("address error");
        assert_eq!(address, "0x45f508caf79cb329a46f1757f3526faf8c6b2ea5");

        let chain_id: u64 = 0;
        let signature = key
            .eth_sign_by_hash(
                "879a053d4800c6354e76c7985a865d2922c82fb5b3f4577b2fe08b998954f2e0".to_owned(),
                chain_id,
            )
            .unwrap();

        assert_eq!(signature.to_string().as_str(),"59e8f544fdee652ae4475a53921ad8030794df66aedf77b218349ba1f476712739caf09dfee2c8ac60e17cc5f2102c09d4ad04de6223a38e9705b28276d71f471b");

        let test_msg = "Example `personal_sign` message";
        let signature = key.eth_sign(test_msg.as_bytes(), chain_id).unwrap();
        assert_eq!(signature.to_string().as_str(),"1490cd65cdfd5145a2b4e4e562b8c78008cb374ac36b2bbcd6b65dbcc14d31c453c705c4399e745fbf22ccd3939754ff2e4bbbe13a7dacae8a44aeb95f6e68c81b");
    }
}
