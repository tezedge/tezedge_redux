pub mod dns_service;
pub use dns_service::{DnsService, DnsServiceDefault};

pub mod randomness_service;
pub use randomness_service::{RandomnessService, RandomnessServiceDefault};

pub mod mio_service;
pub use mio_service::{MioService, MioServiceDefault};

pub trait Service {
    type Randomness: RandomnessService;
    type Dns: DnsService;
    type Mio: MioService;

    fn randomness(&mut self) -> &mut Self::Randomness;

    fn dns(&mut self) -> &mut Self::Dns;

    fn mio(&mut self) -> &mut Self::Mio;
}

pub struct ServiceDefault {
    pub randomness: RandomnessServiceDefault,
    pub dns: DnsServiceDefault,
    pub mio: MioServiceDefault,
}

impl Service for ServiceDefault {
    type Randomness = RandomnessServiceDefault;
    type Dns = DnsServiceDefault;
    type Mio = MioServiceDefault;

    fn randomness(&mut self) -> &mut Self::Randomness {
        &mut self.randomness
    }

    fn dns(&mut self) -> &mut Self::Dns {
        &mut self.dns
    }

    fn mio(&mut self) -> &mut Self::Mio {
        &mut self.mio
    }
}
