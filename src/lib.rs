#![deny(missing_docs)]

//! SSH utilities.
//!
//! ```rust
//! use nbssh::{Address, SshParams};
//! use std::process::Command;
//!
//! let params = SshParams {
//!   address: Address::from_host("myHost"),
//!   ..Default::default()
//! };
//! let args = params.command(&["echo", "hello"]);
//! Command::new(&args[0]).args(&args[1..]).status().unwrap();
//! ```

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display};
use std::path::PathBuf;

/// Default SSH port number 22.
pub const DEFAULT_SSH_PORT: u16 = 22;

/// Host and port number.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Address {
    /// Host name or IP address.
    pub host: String,
    /// Port number.
    pub port: Option<u16>,
}

/// Address parse errors.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AddressError {
    /// The address either contains more than one colon or is empty.
    #[error("invalid address format")]
    InvalidFormat,

    /// The port number could not be parsed as a u16.
    #[error("invalid address port")]
    InvalidPort,
}

impl Address {
    /// Create a new address.
    pub fn new(host: &str, port: u16) -> Address {
        Address {
            host: host.to_string(),
            port: Some(port),
        }
    }

    /// Create a new address with no port number set.
    pub fn from_host(host: &str) -> Address {
        Address {
            host: host.to_string(),
            port: None,
        }
    }

    /// Parse an address in "host[:port]" format.
    pub fn parse(address: &str) -> Result<Address, AddressError> {
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() == 2 {
            if let Ok(port) = parts[1].parse() {
                Ok(Address::new(parts[0], port))
            } else {
                Err(AddressError::InvalidPort)
            }
        } else if parts.len() == 1 {
            Ok(Address::from_host(address))
        } else {
            Err(AddressError::InvalidFormat)
        }
    }

    /// Get the port number as a string.
    pub fn port_str(&self) -> String {
        if let Some(port) = self.port {
            port.to_string()
        } else {
            String::new()
        }
    }
}

struct AddressVisitor;

impl<'de> de::Visitor<'de> for AddressVisitor {
    type Value = Address;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("host[:port]")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Address::parse(value) {
            Ok(addr) => Ok(addr),
            Err(AddressError::InvalidFormat) => {
                Err(E::custom("invalid address format"))
            }
            Err(AddressError::InvalidPort) => {
                Err(E::custom("invalid port number"))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Address, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AddressVisitor)
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(port) = self.port {
            write!(f, "{}:{}", self.host, port)
        } else {
            write!(f, "{}", self.host)
        }
    }
}

/// Inputs for an SSH command, excluding the remote command itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SshParams {
    /// Target address.
    pub address: Address,

    /// Optional identity path ("-i" option).
    pub identity: Option<PathBuf>,

    /// Target user name.
    pub user: Option<String>,

    /// If false, skip the known-host check and do not add the target
    /// to the known-hosts file. This is useful, for example, with
    /// ephemeral VMs.
    ///
    /// Setting this to false adds these flags:
    /// 1. -oStrictHostKeyChecking=no
    /// 2. -oUserKnownHostsFile=/dev/null
    pub strict_host_key_checking: bool,
}

impl Default for SshParams {
    fn default() -> SshParams {
        SshParams {
            address: Address::default(),
            identity: None,
            user: None,
            strict_host_key_checking: true,
        }
    }
}

impl SshParams {
    /// Create a full SSH command.
    pub fn command<S: AsRef<OsStr>>(&self, args: &[S]) -> Vec<OsString> {
        let mut output: Vec<OsString> = Vec::new();
        output.push("ssh".into());

        if !self.strict_host_key_checking {
            output.extend_from_slice(&[
                "-oStrictHostKeyChecking=no".into(),
                "-oUserKnownHostsFile=/dev/null".into(),
            ]);
        }
        output.push("-oBatchMode=yes".into());

        if let Some(identity) = &self.identity {
            output.extend_from_slice(&["-i".into(), identity.into()]);
        }

        if let Some(port) = self.address.port {
            output.extend_from_slice(&["-p".into(), port.to_string().into()]);
        }

        let target = if let Some(user) = &self.user {
            format!("{}@{}", user, self.address.host)
        } else {
            self.address.host.clone()
        };

        output.push(target.into());
        output.extend(args.iter().map(|arg| arg.into()));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};
    use std::path::Path;

    #[test]
    fn test_address_parse() {
        assert_eq!(Address::parse("a"), Ok(Address::from_host("a")));
        assert_eq!(Address::parse("a:1234"), Ok(Address::new("a", 1234)));
        assert_eq!(Address::parse("a:b"), Err(AddressError::InvalidPort));
        assert_eq!(
            Address::parse("a:1234:5678"),
            Err(AddressError::InvalidFormat)
        );
    }

    #[test]
    fn test_address_display() {
        let addr = Address::from_host("abc");
        assert_eq!(format!("{}", addr), "abc");
        let addr = Address::new("abc", 123);
        assert_eq!(format!("{}", addr), "abc:123");
    }

    #[test]
    fn test_address_tokens() {
        assert_tokens(&Address::from_host("abc"), &[Token::Str("abc")]);
        assert_tokens(&Address::new("abc", 123), &[Token::Str("abc:123")]);
    }

    #[test]
    fn test_command() {
        let target = SshParams {
            address: Address::parse("localhost:9222").unwrap(),
            identity: Some(Path::new("/myIdentity").to_path_buf()),
            user: Some("me".to_string()),
            strict_host_key_checking: false,
        };
        let cmd = target.command(&["arg1", "arg2"]);
        assert_eq!(
            cmd,
            vec![
                "ssh",
                "-oStrictHostKeyChecking=no",
                "-oUserKnownHostsFile=/dev/null",
                "-oBatchMode=yes",
                "-i",
                "/myIdentity",
                "-p",
                "9222",
                "me@localhost",
                "arg1",
                "arg2"
            ]
        );
    }
}
