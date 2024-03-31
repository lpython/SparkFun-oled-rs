# STM32 F3 Discovery 
## Rust example for the magnetometer  

Simple example based of the [F3 Discovery Book](https://docs.rust-embedded.org/discovery/f3discovery/) and this [github discussion](https://github.com/rust-embedded/discovery/issues/274).
Removes the STM32 Discovery Board crate and uses the [lsm303agr library](https://crates.io/crates/lsm303agr).

### Note : This was developed on the F303C-E02 board revision

### Usage
ITM must be setup [LINK](https://docs.rust-embedded.org/discovery/f3discovery/06-hello-world/index.html) or it will hang.

