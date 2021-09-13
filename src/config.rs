use shell_state::ShellCompatibilityVersion;
use tezos_identity::Identity;

use crate::Port;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: Port,
    pub disable_mempool: bool,
    pub private_node: bool,
    pub pow_target: f64,
    pub identity: Identity,
    pub shell_compatibility_version: ShellCompatibilityVersion,
}

pub fn default_config() -> Config {
    let pow_target = 26.0;
    Config {
        port: 9732,
        disable_mempool: false,
        private_node: false,
        pow_target,
        identity: Identity::generate(pow_target).unwrap(),
        shell_compatibility_version: ShellCompatibilityVersion::new(
            "TEZOS_MAINNET".to_owned(),
            vec![0, 1],
            vec![1],
        ),
    }
}
