#include "chainmain.h"
#include "cronos.h"
#include "wallet.h"
#include "sdk/include/defi-wallet-core-cpp/src/lib.rs.h"
#include "sdk/include/defi-wallet-core-cpp/src/nft.rs.h"
#include "sdk/include/defi-wallet-core-cpp/src/ethereum.rs.h"
#include "sdk/include/rust/cxx.h"
#include <atomic>
#include <cassert>
#include <chrono>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <thread>
using namespace std;
using namespace rust;
using namespace org::defi_wallet_core;
Box<Wallet> createWallet(String mymnemonics);
inline String getEnv(String key) {
  String ret;
  if (getenv(key.c_str()) != nullptr) {
    ret = getenv(key.c_str());
  }
  return ret;
}

//  read bytes from char array and create hexstring
std::string bytes_to_hexstring(const char *bytes, size_t len) {
  ostringstream os;
  os << hex << setfill('0');
  for (size_t i = 0; i < len; ++i) {
    os << setw(2) << static_cast<unsigned int>(static_cast<unsigned char>(bytes[i]));
  }
  return os.str();
}




int main(int argc, char *argv[]) {
  // read file MyErc721.json to string
  ifstream t("erc721-abi.json");
  stringstream buffer;
  buffer << t.rdbuf();
  std::string json = buffer.str();
  // print json
  //cout << json << endl;
  // create wallet
  /*
  try {
    org::defi_wallet_core::set_cronos_httpagent("cronos-wallet-cpp-example");
    test_wallet();
    chainmain_process();   // chain-main
    test_chainmain_nft();  // chainmain nft tests
    test_login();          // decentralized login
    cronos_process();      // cronos
    test_cronos_testnet(); // cronos testnet
  } catch (const rust::cxxbridge1::Error &e) {
    // Use `Assertion failed`, the same as `assert` function
    std::cout << "Assertion failed: " << e.what() << std::endl;
  }
  // read env MYSENDER to string
  

  test_interval();*/

  
  
  String mymnemonics = getEnv("MYMNEMONICS2");
  String mycronosrpc = getEnv("MYCRONOSRPC2");
  Box<Wallet> mywallet = createWallet(mymnemonics);

  String  mySender = mywallet->get_eth_address(0);
  String myReceiver = mywallet->get_eth_address(2);
  // print mySender
  cout << "get_eth_nonce  from=" << mySender << endl;
  // print mycronosrpc
  cout << "mycronosrpc " << mycronosrpc << endl;
  auto nonce1 = get_eth_nonce(mySender.c_str(), mycronosrpc);
  // display nonce
  cout << "nonce1 " << nonce1 << endl;

  


  // print mySender
  cout << "mySender: " << mySender << endl;
  // print myReceiver
  cout << "myReceiver: " << myReceiver << endl;
  Box<EthContract> w = new_eth_contract(json);
  w->add_address(mySender); // from 
  w->add_address(myReceiver); // to
  w->add_uint("2"); // tokenId
   Vec<uint8_t> data=w->encode("safeTransferFrom"); // encoded 
  std::string hexstring = bytes_to_hexstring((char *)data.data(), data.size());
  // print data length
  cout << "data length: " << data.size() << endl;
  cout<<hexstring<<endl;
  
  rust::String a=w->test();

  char hdpath[100];
  int cointype = 60;
  int chainid = 1; // defined in cronos-devnet.yaml
  snprintf(hdpath, sizeof(hdpath), "m/44'/%d'/0'/0/0", cointype);
  Box<PrivateKey> privatekey = mywallet->get_key(hdpath);


  EthTxInfoRaw eth_tx_info = new_eth_tx_info();
  eth_tx_info.to_address = myReceiver.c_str();
  eth_tx_info.nonce = nonce1;
  eth_tx_info.amount = "0";
  eth_tx_info.amount_unit = EthAmount::EthDecimal;
  eth_tx_info.data= data;
  eth_tx_info.gas_limit="219400";
  eth_tx_info.gas_price="100000000";
  eth_tx_info.gas_price_unit = EthAmount::WeiDecimal;


  Vec<uint8_t> signedtx =
      build_eth_signed_tx(eth_tx_info, chainid, true, *privatekey);
  //U256 balance = get_eth_balance(myaddress1.c_str(), mycronosrpc);
  String status =
      broadcast_eth_signed_raw_tx(signedtx, mycronosrpc, 1000).status;
  // print status
  cout << "status: " << status << endl;

  
  cout<<"hello "<<a.c_str()<<endl;
  return 0;
}
