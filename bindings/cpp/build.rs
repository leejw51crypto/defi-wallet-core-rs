const BRIDGES: &[&str] = &[
    "src/lib.rs",
    "src/nft.rs",
    "src/contract.rs",
    "src/ethereum.rs",
    "src/uint.rs",
];



#[cfg(target_os = "android")]
fn main() {
    cxx_build::CFG.doxygen = true;
    cxx_build::bridges(BRIDGES)
        .file("src/nft.cc")
        .file("src/android.cc")
        .flag_if_supported("-std=c++11")
        .compile("defi_wallet_core");

    for bridge in BRIDGES {
        println!("cargo:rerun-if-changed={}", bridge);
    }

    println!("cargo:rerun-if-changed=src/nft.cc");
    println!("cargo:rerun-if-changed=include/nft.h");
}


#[cfg(not(target_os = "android"))]
fn main() {
    cxx_build::CFG.doxygen = true;
    cxx_build::bridges(BRIDGES)
        .file("src/nft.cc")
        .flag_if_supported("-std=c++11")
        .compile("defi_wallet_core");

    for bridge in BRIDGES {
        println!("cargo:rerun-if-changed={}", bridge);
    }

    println!("cargo:rerun-if-changed=src/nft.cc");
    println!("cargo:rerun-if-changed=include/nft.h");
}
