/// Cryptographic operations and abstractions
use crate::error::Result;

// Conditional crypto imports - prioritize ring if both features are enabled
#[cfg(all(feature = "ring-crypto", not(feature = "aws-lc-crypto")))]
use ring::rand::SecureRandom;
#[cfg(all(feature = "ring-crypto", not(feature = "aws-lc-crypto")))]
use ring::{aead, digest, pbkdf2, rand};

#[cfg(all(feature = "aws-lc-crypto", not(feature = "ring-crypto")))]
use aws_lc_rs::rand::SecureRandom;
#[cfg(all(feature = "aws-lc-crypto", not(feature = "ring-crypto")))]
use aws_lc_rs::{aead, digest, pbkdf2, rand};

// If both features are enabled, prefer ring (for CI --all-features)
#[cfg(all(feature = "ring-crypto", feature = "aws-lc-crypto"))]
use ring::rand::SecureRandom;
#[cfg(all(feature = "ring-crypto", feature = "aws-lc-crypto"))]
use ring::{aead, digest, pbkdf2, rand};

pub mod tls;

/// Cryptographic engine for VPN operations
pub struct CryptoEngine {
    rng: rand::SystemRandom,
}

impl CryptoEngine {
    /// Create a new crypto engine
    pub fn new() -> Result<Self> {
        Ok(Self {
            rng: rand::SystemRandom::new(),
        })
    }

    /// Encrypt data using AES-GCM
    pub fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(crate::error::VpnError::Network(
                "Key must be 32 bytes for AES-256".into(),
            ));
        }

        let key =
            aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, key).map_err(
                |e| crate::error::VpnError::Network(format!("Key creation failed: {e:?}")),
            )?);

        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes).map_err(|e| {
            crate::error::VpnError::Network(format!("Nonce generation failed: {e:?}"))
        })?;

        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = data.to_vec();
        key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|e| crate::error::VpnError::Network(format!("Encryption failed: {e:?}")))?;

        // Prepend nonce to encrypted data
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);
        Ok(result)
    }

    /// Decrypt data using AES-GCM
    pub fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(crate::error::VpnError::Network(
                "Key must be 32 bytes for AES-256".into(),
            ));
        }

        if data.len() < 12 {
            return Err(crate::error::VpnError::Network(
                "Data too short to contain nonce".into(),
            ));
        }

        let key =
            aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, key).map_err(
                |e| crate::error::VpnError::Network(format!("Key creation failed: {e:?}")),
            )?);

        let (nonce_bytes, encrypted_data) = data.split_at(12);
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|e| crate::error::VpnError::Network(format!("Invalid nonce: {e:?}")))?;

        let mut in_out = encrypted_data.to_vec();
        let decrypted = key
            .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|e| crate::error::VpnError::Network(format!("Decryption failed: {e:?}")))?;

        Ok(decrypted.to_vec())
    }

    /// Generate random bytes
    pub fn random_bytes(&self, length: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0u8; length];
        self.rng.fill(&mut bytes).map_err(|e| {
            crate::error::VpnError::Network(format!("Random generation failed: {e:?}"))
        })?;
        Ok(bytes)
    }

    /// Compute SHA-256 hash
    pub fn hash(&self, data: &[u8]) -> Result<Vec<u8>> {
        let hash = digest::digest(&digest::SHA256, data);
        Ok(hash.as_ref().to_vec())
    }

    /// Derive key using PBKDF2
    pub fn derive_key(&self, password: &[u8], salt: &[u8], iterations: u32) -> Result<Vec<u8>> {
        let mut key = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(iterations)
                .unwrap_or(std::num::NonZeroU32::new(1000).unwrap()),
            salt,
            password,
            &mut key,
        );
        Ok(key.to_vec())
    }
}

impl Default for CryptoEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default crypto engine")
    }
}
