use std::{fmt::Debug, net::SocketAddr};

use crypto::nonce::Nonce;
use rand::Rng;

pub type RandomnessServiceDefault = rand::rngs::ThreadRng;

pub trait RandomnessService {
    fn get_nonce(&mut self, peer: SocketAddr) -> Nonce;
}

impl<R> RandomnessService for R
where
    R: Rng + Debug,
{
    fn get_nonce(&mut self, _: SocketAddr) -> Nonce {
        let mut b = [0; 24];
        self.fill(&mut b);
        Nonce::new(&b)
    }
}
