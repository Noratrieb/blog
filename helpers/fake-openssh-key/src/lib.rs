use cluelessh_keys::{
    private::{KeyEncryptionParams, PlaintextPrivateKey, PrivateKey},
    public::{PublicKey, PublicKeyWithComment},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_fake(public_key: &str) -> Result<String, String> {
    let public_key = public_key
        .parse::<PublicKeyWithComment>()
        .map_err(|err| format!("invalid public key: {err}"))?;

    let mut fake_private_key = PlaintextPrivateKey::generate(
        "".into(),
        cluelessh_keys::KeyGenerationParams {
            key_type: match public_key.key {
                PublicKey::Ed25519 { .. } => cluelessh_keys::KeyType::Ed25519,
                PublicKey::EcdsaSha2NistP256 { .. } => cluelessh_keys::KeyType::Ecdsa,
            },
        },
    );

    match public_key.key {
        PublicKey::Ed25519 { public_key } => {
            let PrivateKey::Ed25519 {
                public_key: fake_public_key,
                ..
            } = &mut fake_private_key.private_key
            else {
                panic!()
            };
            *fake_public_key = public_key;
        }
        PublicKey::EcdsaSha2NistP256 { public_key } => {
            let PrivateKey::EcdsaSha2NistP256 {
                public_key: fake_public_key,
                ..
            } = &mut fake_private_key.private_key
            else {
                panic!()
            };
            *fake_public_key = public_key;
        }
    }

    let fake_private_key = fake_private_key
        .encrypt(KeyEncryptionParams::plaintext())
        .map_err(|err| format!("failed to encode key: {err}"))?;

    Ok(fake_private_key.to_bytes_armored())
}
