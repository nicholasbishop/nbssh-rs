use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

impl Address {
    pub fn new(host: &str, port: u16) -> Address {
        Address {
            host: host.to_string(),
            port,
        }
    }

    pub fn parse(address: &str) -> Option<Address> {
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() == 2 {
            if let Ok(port) = parts[1].parse() {
                Some(Address::new(parts[0], port))
            } else {
                None
            }
        } else if parts.len() == 1 {
            Some(Address::new(address, 22))
        } else {
            None
        }
    }

    pub fn port_str(&self) -> String {
        self.port.to_string()
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

struct AddressVisitor;

impl<'de> Visitor<'de> for AddressVisitor {
    type Value = Address;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("host[:port]")
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

    #[test]
    fn test_parse() {
        assert_eq!(Address::parse("a").unwrap(), Address::new("a", 22));
        assert_eq!(Address::parse("a:1234").unwrap(), Address::new("a", 1234));
        assert!(Address::parse("a:b").is_none());
        assert!(Address::parse("a:1234:5678").is_none());
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
}
