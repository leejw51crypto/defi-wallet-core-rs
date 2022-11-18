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

  test_interval();*/
  Box<EthContract> w = new_eth_contract(json);
  w->add_address("0x8c8bdBe9CeE455732525086264a4Bf9Cf821C498"); // from 
  w->add_address("0x8c8bdBe9CeE455732525086264a4Bf9Cf821C498"); // to
  w->add_uint("0"); // tokenId
   Vec<uint8_t> data=w->encode("safeTransferFrom"); // encoded 
  std::string hexstring = bytes_to_hexstring((char *)data.data(), data.size());
  // print data length
  cout << "data length: " << data.size() << endl;
  cout<<hexstring<<endl;
  
  rust::String a=w->test();

  
  cout<<"hello "<<a.c_str()<<endl;
  return 0;
}
