use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AddressError {
    InvalidFormat,
    InvalidPort,
}

impl Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddressError::InvalidFormat => write!(f, "invalid address format"),
            AddressError::InvalidPort => write!(f, "invalid address port"),
        }
    }
}

impl Error for AddressError {}

impl Address {
    pub fn new(host: &str, port: u16) -> Address {
        Address {
            host: host.to_string(),
            port,
        }
    }

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

    pub fn parse_vec(addresses: &[String]) -> Result<Vec<Address>, AddressError> {
        let mut result = Vec::new();
        for elem in addresses {
            match Address::parse(elem) {
                Ok(addr) => result.push(addr),
                Err(err) => return Err(err),
            }
        }
        Ok(result)
    }

    pub fn port_str(&self) -> String {
        self.port.to_string()
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
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
            Err(AddressError::InvalidFormat) => Err(E::custom("invalid address format")),
            Err(AddressError::InvalidPort) => Err(E::custom("invalid port number")),
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
        write!(f, "{}:{}", self.host, self.port)
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
    fn test_parse_vec() {
        assert_eq!(
            Address::parse_vec(&["a".to_string(), "b:9222".to_string()]),
            Ok(vec![Address::new("a", 22), Address::new("b", 9222)])
        );

        assert_eq!(
            Address::parse_vec(&["a".to_string(), "b:abcd".to_string()]),
            Err(AddressError::InvalidPort)
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Address::new("abc", 123).to_string(), "abc:123");
    }

    #[test]
    fn test_display() {
        let addr = Address::new("abc", 22);
        assert_eq!(format!("{}", addr), "abc:22");
    }

    #[test]
    fn test_ser_de() {
        let addr = Address::new("abc", 22);
        assert_tokens(&addr, &[Token::Str("abc:22")]);
    }

    #[test]
    fn test_error_trait() {
        let err: Box<Error> = Box::new(AddressError::InvalidPort);
        assert!(err.cause().is_none());
    }
}
