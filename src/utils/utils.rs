use bytes::Bytes;

// Implementation of the cryptographic hasher using SHA-256
#[derive(Clone)]
pub struct Sha256Hasher {
    state: Vec<u8>
}

impl commonware_cryptography::Hasher for Sha256Hasher {
    fn new() -> Self {
        Self {
            state: Vec::new()
        }
    }

    fn update(&mut self, message: &[u8]) {
        self.state.extend_from_slice(message);
    }

    fn finalize(&mut self) -> Bytes {
        let result = commonware_utils::hash(&self.state);
        self.reset();
        Bytes::from(result)
    }

    fn reset(&mut self) {
        self.state.clear();
    }

    fn validate(digest: &Bytes) -> bool {
        digest.len() == 32
    }

    fn len() -> usize {
        32
    }

    fn random<R: rand::Rng + rand::CryptoRng>(rng: &mut R) -> Bytes {
        let mut bytes = vec![0u8; 32];
        rng.fill_bytes(&mut bytes);
        Bytes::from(bytes)
    }
}