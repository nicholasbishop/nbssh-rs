# nbssh

[![crates.io](https://img.shields.io/crates/v/nbssh.svg)](https://crates.io/crates/nbssh)
[![Documentation](https://docs.rs/nbssh/badge.svg)](https://docs.rs/nbssh)

SSH command generator. Example usage:

```rust
use nbssh::{Address, SshParams};
use std::process::Command;

let params = SshParams {
  address: Address::from_host("myHost"),
  ..Default::default()
};
let args = params.command(&["echo", "hello"]);
Command::new(&args[0]).args(&args[1..]).status().unwrap();
```
