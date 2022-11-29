#include "chainmain.h"
#include "cronos.h"
#include "wallet.h"
#include "sdk/include/defi-wallet-core-cpp/src/lib.rs.h"
#include "sdk/include/defi-wallet-core-cpp/src/nft.rs.h"
#include "sdk/include/rust/cxx.h"
#include <atomic>
#include <cassert>
#include <chrono>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <thread>

void test_dynamic_deploy();
void test_dynamic_mint(std::string usercontract) ;

void test_dynamic_api_send();
void test_dynamic_api_encode();
void test_dynamic_api_call();
int main(int argc, char *argv[]) {
  try {
    test_dynamic_deploy();
    //test_dynamic_mint("");

    //test_dynamic_api_call();
    //test_dynamic_deploy();
    //test_dynamic_minting_api_call();
    // test_dynamic_api_encode();
    // test_dynamic_api_send();

    //test_dynamic_minting_api_send_deploy();
    //test_dynamic_minting_api_send();
    return 0;

    org::defi_wallet_core::set_cronos_httpagent("cronos-wallet-cpp-example");
    test_wallet();
    chainmain_process();   // chain-main
    test_chainmain_nft();  // chainmain nft tests
    test_login();          // decentralized login
    cronos_process();      // cronos
    test_cronos_testnet(); // cronos testnet
    test_interval();
  } catch (const rust::cxxbridge1::Error &e) {
    // Use `Assertion failed`, the same as `assert` function
    std::cout << "Assertion failed: " << e.what() << std::endl;
  }
  return 0;
}
