use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display};
use std::path::PathBuf;

pub const DEFAULT_SSH_PORT: u16 = 22;

/// Host and port number.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Address {
    /// Host name or IP address.
    pub host: String,
    /// Port number.
    pub port: u16,
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
            port,
        }
    }

    /// Parse an address in "host[:port]" format. If port is not
    /// given, it defaults to 22.
    pub fn parse(address: &str) -> Result<Address, AddressError> {
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() == 2 {
            if let Ok(port) = parts[1].parse() {
                Ok(Address::new(parts[0], port))
            } else {
                Err(AddressError::InvalidPort)
            }
        } else if parts.len() == 1 {
            Ok(Address::new(address, DEFAULT_SSH_PORT))
        } else {
            Err(AddressError::InvalidFormat)
        }
    }

    /// Get the port number as a string.
    pub fn port_str(&self) -> String {
        self.port.to_string()
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
        if self.port == 22 {
            write!(f, "{}", self.host)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

/// Inputs for an SSH command, excluding the remote command itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SshParams {
    /// Target address.
    pub address: Address,

    /// Identity file ("-i" option).
    pub identity: PathBuf,

    /// Target user name.
    pub user: String,

    /// If false, skip the known-host check and do not add the target
    /// to the known-hosts file. This is useful, for example, with
    /// ephemeral VMs.
    ///
    /// Setting this to false adds these flags:
    /// 1. -oStrictHostKeyChecking=no
    /// 2. -oUserKnownHostsFile=/dev/null
    pub strict_host_key_checking: bool,
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
        output.extend_from_slice(&[
            "-oBatchMode=yes".into(),
            "-i".into(),
            self.identity.clone().into(),
            "-p".into(),
            self.address.port_str().into(),
            format!("{}@{}", self.user, self.address.host).into(),
        ]);
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
        assert_eq!(Address::parse("a"), Ok(Address::new("a", 22)));
        assert_eq!(Address::parse("a:1234"), Ok(Address::new("a", 1234)));
        assert_eq!(Address::parse("a:b"), Err(AddressError::InvalidPort));
        assert_eq!(
            Address::parse("a:1234:5678"),
            Err(AddressError::InvalidFormat)
        );
    }

    #[test]
    fn test_address_display() {
        let addr = Address::new("abc", 22);
        assert_eq!(format!("{}", addr), "abc");
        let addr = Address::new("abc", 123);
        assert_eq!(format!("{}", addr), "abc:123");
    }

    #[test]
    fn test_address_deserialize() {
        let addr = Address::new("abc", 22);
        assert_tokens(&addr, &[Token::Str("abc")]);
    }

    #[test]
    fn test_command() {
        let address = Address::parse("localhost:9222").unwrap();
        let identity = Path::new("/myIdentity").to_path_buf();
        let user = "me";
        let target = SshParams {
            address,
            identity,
            user: user.into(),
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
