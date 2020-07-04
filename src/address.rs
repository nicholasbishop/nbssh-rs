use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Display;

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
            Ok(Address::new(address, 22))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_parse() {
        assert_eq!(Address::parse("a"), Ok(Address::new("a", 22)));
        assert_eq!(Address::parse("a:1234"), Ok(Address::new("a", 1234)));
        assert_eq!(Address::parse("a:b"), Err(AddressError::InvalidPort));
        assert_eq!(
            Address::parse("a:1234:5678"),
            Err(AddressError::InvalidFormat)
        );
    }

    #[test]
    fn test_display() {
        let addr = Address::new("abc", 22);
        assert_eq!(format!("{}", addr), "abc");
        let addr = Address::new("abc", 123);
        assert_eq!(format!("{}", addr), "abc:123");
    }

    #[test]
    fn test_ser_de() {
        let addr = Address::new("abc", 22);
        assert_tokens(&addr, &[Token::Str("abc")]);
    }
}
