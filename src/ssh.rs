use crate::address::Address;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug)]
pub struct SshTarget {
    address: Address,
    identity: PathBuf,
    user: String,
}

impl SshTarget {
    pub fn new(address: Address, identity: PathBuf, user: &str) -> SshTarget {
        SshTarget {
            address,
            identity,
            user: user.to_string(),
        }
    }

    pub fn command<S: AsRef<OsStr>>(&self, args: &[S]) -> subprocess::Exec {
        subprocess::Exec::cmd("ssh")
            .args(&[
                "-oStrictHostKeyChecking=no",
                "-oUserKnownHostsFile=/dev/null",
                "-oBatchMode=yes",
                "-i",
                self.identity.to_str().unwrap(),
                "-p",
                &self.address.port_str(),
                &format!("{}@{}", self.user, self.address.host),
            ])
            .args(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_command() {
        let address = Address::parse("localhost:9222").unwrap();
        let identity = Path::new("/myIdentity").to_path_buf();
        let user = "me";
        let target = SshTarget::new(address, identity, user);
        let cmd = target.command(&["arg1", "arg2"]);
        assert_eq!(cmd.to_cmdline_lossy(), "ssh '-oStrictHostKeyChecking=no' '-oUserKnownHostsFile=/dev/null' '-oBatchMode=yes' -i /myIdentity -p 9222 'me@localhost' arg1 arg2");
    }
}
