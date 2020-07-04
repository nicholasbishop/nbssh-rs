use crate::address::Address;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug)]
pub struct SshTarget {
    pub address: Address,
    pub identity: PathBuf,
    pub user: String,
    pub strict_host_key_checking: bool,
}

impl SshTarget {
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
    use std::path::Path;

    #[test]
    fn test_command() {
        let address = Address::parse("localhost:9222").unwrap();
        let identity = Path::new("/myIdentity").to_path_buf();
        let user = "me";
        let target = SshTarget {
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
