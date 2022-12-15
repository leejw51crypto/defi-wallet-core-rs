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

void test_dynamic_mint();
void test_dynamic_mint_with_encoding();
int main(int argc, char *argv[]) {
  try {
    test_dynamic_mint_with_encoding();
    //test_dynamic_mint();  
  } catch (const rust::cxxbridge1::Error &e) {
    // Use `Assertion failed`, the same as `assert` function
    std::cout << "Assertion failed: " << e.what() << std::endl;
  }

  return 0;
}
