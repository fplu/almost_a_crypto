use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};

pub trait Signable {
    fn bytes(&self) -> Vec<u8>;

    fn sign(&self, keypair: &Keypair) -> Signature {
        keypair.sign(&self.bytes())
    }

    fn verify(&self, public_key: PublicKey, signature: Signature) -> bool {
        public_key.verify(&self.bytes(), &signature).is_ok()
    }
}
