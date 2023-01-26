#include "rust/cxx.h"
namespace org {
namespace defi_wallet_core {
    int secureStorageWrite(rust::String userkey, rust::String uservalue);
    rust::String secureStorageRead(rust::String userkey);
} // namespace defi_wallet_core
} // namespace org