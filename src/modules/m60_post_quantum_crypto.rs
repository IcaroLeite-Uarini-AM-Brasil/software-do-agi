//! Troca de chaves ML-KEM-1024 via implementação auditada da família pqcrypto.
//! Este módulo implementa encapsulamento de chaves, não assinaturas. Para
//! assinatura pós-quântica, use um algoritmo separado, como ML-DSA, conforme
//! a política criptográfica do projeto. Consulte a documentação antes de uso
//! em produção; a escolha da implementação e a gestão das chaves também importam.

use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{Ciphertext as _, PublicKey as _, SecretKey as _, SharedSecret as _};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    InvalidPublicKey,
    InvalidSecretKey,
    InvalidCiphertext,
}

/// Par ML-KEM-1024. A chave secreta não é exibida em Debug.
pub struct MlKemKeypair {
    public_key: kyber1024::PublicKey,
    secret_key: kyber1024::SecretKey,
}

impl std::fmt::Debug for MlKemKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MlKemKeypair")
            .field("public_key_bytes", &self.public_key.as_bytes().len())
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

impl MlKemKeypair {
    pub fn public_key_bytes(&self) -> &[u8] {
        self.public_key.as_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encapsulation {
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PostQuantumEngine;

impl PostQuantumEngine {
    pub const ALGORITHM_NAME: &'static str = "ML-KEM-1024 (FIPS 203)";

    pub fn new() -> Self {
        Self
    }

    pub fn generate_keypair(&self) -> MlKemKeypair {
        let (public_key, secret_key) = kyber1024::keypair();
        MlKemKeypair { public_key, secret_key }
    }

    pub fn encapsulate(&self, public_key: &[u8]) -> Result<Encapsulation, CryptoError> {
        let public_key = kyber1024::PublicKey::from_bytes(public_key)
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let (shared_secret, ciphertext) = kyber1024::encapsulate(&public_key);
        Ok(Encapsulation {
            ciphertext: ciphertext.as_bytes().to_vec(),
            shared_secret: shared_secret.as_bytes().to_vec(),
        })
    }

    pub fn decapsulate(&self, keypair: &MlKemKeypair, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let ciphertext = kyber1024::Ciphertext::from_bytes(ciphertext)
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        let shared_secret = kyber1024::decapsulate(&ciphertext, &keypair.secret_key);
        Ok(shared_secret.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encapsulation_and_decapsulation_agree() {
        let engine = PostQuantumEngine::new();
        let keys = engine.generate_keypair();
        let encapsulated = engine.encapsulate(keys.public_key_bytes()).unwrap();
        let recovered = engine.decapsulate(&keys, &encapsulated.ciphertext).unwrap();
        assert_eq!(encapsulated.shared_secret, recovered);
        assert_eq!(recovered.len(), 32);
    }

    #[test]
    fn rejects_malformed_public_key_and_ciphertext() {
        let engine = PostQuantumEngine::new();
        assert_eq!(engine.encapsulate(&[0; 8]), Err(CryptoError::InvalidPublicKey));
        let keys = engine.generate_keypair();
        assert_eq!(engine.decapsulate(&keys, &[0; 8]), Err(CryptoError::InvalidCiphertext));
    }
}
