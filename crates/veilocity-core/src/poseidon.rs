//! Poseidon hash implementation using light-poseidon
//!
//! CRITICAL: This must produce identical outputs to the Noir poseidon library
//! Both use BN254 scalar field with Circom-compatible parameters.

use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};
use light_poseidon::{Poseidon, PoseidonHasher as LightPoseidonHasher};

/// Field element type (BN254 scalar field)
pub type FieldElement = Fr;

/// Poseidon hasher wrapper for consistent API
pub struct PoseidonHasher {
    hasher_1: Poseidon<Fr>,
    hasher_2: Poseidon<Fr>,
    hasher_3: Poseidon<Fr>,
    hasher_4: Poseidon<Fr>,
}

impl PoseidonHasher {
    pub fn new() -> Self {
        Self {
            hasher_1: Poseidon::<Fr>::new_circom(1).expect("Failed to create hasher"),
            hasher_2: Poseidon::<Fr>::new_circom(2).expect("Failed to create hasher"),
            hasher_3: Poseidon::<Fr>::new_circom(3).expect("Failed to create hasher"),
            hasher_4: Poseidon::<Fr>::new_circom(4).expect("Failed to create hasher"),
        }
    }

    /// Hash 1 field element
    pub fn hash1(&mut self, a: &FieldElement) -> FieldElement {
        self.hasher_1.hash(&[*a]).expect("Hash failed")
    }

    /// Hash 2 field elements (for Merkle tree nodes)
    pub fn hash2(&mut self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        self.hasher_2.hash(&[*left, *right]).expect("Hash failed")
    }

    /// Hash 3 field elements (for account leaves)
    pub fn hash3(
        &mut self,
        a: &FieldElement,
        b: &FieldElement,
        c: &FieldElement,
    ) -> FieldElement {
        self.hasher_3.hash(&[*a, *b, *c]).expect("Hash failed")
    }

    /// Hash 4 field elements
    pub fn hash4(
        &mut self,
        a: &FieldElement,
        b: &FieldElement,
        c: &FieldElement,
        d: &FieldElement,
    ) -> FieldElement {
        self.hasher_4.hash(&[*a, *b, *c, *d]).expect("Hash failed")
    }

    /// Derive public key from secret
    pub fn derive_pubkey(&mut self, secret: &FieldElement) -> FieldElement {
        self.hash1(secret)
    }

    /// Compute nullifier for spending
    pub fn compute_nullifier(
        &mut self,
        secret: &FieldElement,
        index: &FieldElement,
        nonce: &FieldElement,
    ) -> FieldElement {
        self.hash3(secret, index, nonce)
    }

    /// Compute account leaf commitment
    pub fn compute_leaf(
        &mut self,
        pubkey: &FieldElement,
        balance: &FieldElement,
        nonce: &FieldElement,
    ) -> FieldElement {
        self.hash3(pubkey, balance, nonce)
    }

    /// Compute deposit commitment
    pub fn compute_deposit_commitment(
        &mut self,
        secret: &FieldElement,
        amount: &FieldElement,
    ) -> FieldElement {
        self.hash2(secret, amount)
    }
}

impl Default for PoseidonHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert field element to bytes (big-endian, 32 bytes)
pub fn field_to_bytes(f: &FieldElement) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let repr = f.into_bigint().to_bytes_be();
    bytes.copy_from_slice(&repr);
    bytes
}

/// Convert bytes to field element (big-endian)
pub fn bytes_to_field(bytes: &[u8; 32]) -> FieldElement {
    FieldElement::from_be_bytes_mod_order(bytes)
}

/// Convert u128 to field element
pub fn u128_to_field(value: u128) -> FieldElement {
    FieldElement::from(value)
}

/// Convert u64 to field element
pub fn u64_to_field(value: u64) -> FieldElement {
    FieldElement::from(value)
}

/// Convert field element to hex string (with 0x prefix)
pub fn field_to_hex(f: &FieldElement) -> String {
    format!("0x{}", hex::encode(field_to_bytes(f)))
}

/// Convert hex string to field element
pub fn hex_to_field(s: &str) -> Result<FieldElement, crate::error::CoreError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).map_err(|_| crate::error::CoreError::InvalidFieldElement)?;
    if bytes.len() != 32 {
        return Err(crate::error::CoreError::InvalidFieldElement);
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(bytes_to_field(&arr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash2_deterministic() {
        let mut hasher = PoseidonHasher::new();
        let a = FieldElement::from(1u64);
        let b = FieldElement::from(2u64);

        let hash1 = hasher.hash2(&a, &b);
        let hash2 = hasher.hash2(&a, &b);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash2_different_inputs() {
        let mut hasher = PoseidonHasher::new();
        let a = FieldElement::from(1u64);
        let b = FieldElement::from(2u64);
        let c = FieldElement::from(3u64);

        let hash1 = hasher.hash2(&a, &b);
        let hash2 = hasher.hash2(&a, &c);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_derive_pubkey() {
        let mut hasher = PoseidonHasher::new();
        let secret = FieldElement::from(12345u64);

        let pubkey = hasher.derive_pubkey(&secret);

        assert_ne!(pubkey, secret);
        assert_ne!(pubkey, FieldElement::from(0u64));
    }

    #[test]
    fn test_compute_leaf() {
        let mut hasher = PoseidonHasher::new();
        let pubkey = FieldElement::from(100u64);
        let balance = u128_to_field(1_000_000_000_000_000_000u128); // 1 MNT
        let nonce = FieldElement::from(0u64);

        let leaf = hasher.compute_leaf(&pubkey, &balance, &nonce);

        assert_ne!(leaf, FieldElement::from(0u64));
    }

    #[test]
    fn test_field_bytes_roundtrip() {
        let original = FieldElement::from(123456789u64);
        let bytes = field_to_bytes(&original);
        let recovered = bytes_to_field(&bytes);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_field_hex_roundtrip() {
        let original = FieldElement::from(987654321u64);
        let hex_str = field_to_hex(&original);
        let recovered = hex_to_field(&hex_str).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_nullifier_uniqueness() {
        let mut hasher = PoseidonHasher::new();
        let secret = FieldElement::from(12345u64);

        let null1 = hasher.compute_nullifier(
            &secret,
            &FieldElement::from(0u64),
            &FieldElement::from(0u64),
        );
        let null2 = hasher.compute_nullifier(
            &secret,
            &FieldElement::from(0u64),
            &FieldElement::from(1u64),
        );
        let null3 = hasher.compute_nullifier(
            &secret,
            &FieldElement::from(1u64),
            &FieldElement::from(0u64),
        );

        assert_ne!(null1, null2);
        assert_ne!(null1, null3);
        assert_ne!(null2, null3);
    }
}
