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
using namespace org::defi_wallet_core;
using namespace std;
using namespace rust;

int main(int argc, char *argv[]) {
  try {
    // read env varialbe MYMNEMONICS
    std::string mymnemonics = std::getenv("MYMNEMONICS");
    rust::Box<Wallet> w2=restore_wallet_save_to_securestorage(mymnemonics,"", "myservice","myapp2");
    rust::Box<Wallet> w=restore_wallet_load_from_securestorage("myservice","myapp2");
    // get address from w
    std::string myaddress = w->get_eth_address(0).c_str();
    // print myaddress
    std::cout<<"myaddress:"<<myaddress<<std::endl;


    
  } catch (const rust::cxxbridge1::Error &e) {
    // Use `Assertion failed`, the same as `assert` function
    std::cout << "Assertion failed: " << e.what() << std::endl;
  }

  return 0;
}
