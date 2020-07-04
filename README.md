# nbssh

[![Build Status](https://travis-ci.org/nicholasbishop/nbssh-rs.svg?branch=master)](https://travis-ci.org/nicholasbishop/nbssh-rs)
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
