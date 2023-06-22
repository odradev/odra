use libsecp256k1::Error;
use sha3::{Digest, Keccak256};

pub(crate) fn keccak256(input: &str) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let result = result.as_slice();
    result.try_into().unwrap()
}

/// This function is used to recover the public key from the signature.
/// The signature needs to be in libsecp256k1 format.
/// hash is the hash of the message that was signed.
/// recovery_id is the recovery id of the signature (0 to 3).
#[allow(dead_code)]
pub fn ecrecover(
    hash: &[u8; 32],
    signature: &[u8; 64],
    recovery_id: u8
) -> Result<libsecp256k1::PublicKey, Error> {
    let message = libsecp256k1::Message::parse_slice(hash)?;
    let recovery_id = libsecp256k1::RecoveryId::parse(recovery_id)?;
    let signature = libsecp256k1::Signature::parse_standard_slice(signature)?;
    let public_key = libsecp256k1::recover(&message, &signature, &recovery_id)?;
    Ok(public_key)
}

#[cfg(test)]
mod test {
    use super::keccak256;
    use crate::crypto::ecrecover;
    use hex::ToHex;
    use odra::{contract_env, test_env};
    use odra::types::Bytes;
    use sp_io::hashing::sha2_256;

    const ADMIN: &str = "f23ec0bb4210edd5cba85afd05127efcd2fc6a781bfed49188da1081670b22d8";
    const KRZYSZTOF: &str = "08bad378ceebd5f420e3050c73e6aa8b7da47a744912ec7494e32675f839114c";

    #[test]
    fn keccak256_works() {
        let krzysztof = keccak256("krzysztof");
        assert_eq!(krzysztof.encode_hex::<String>(), KRZYSZTOF);

        let admin = keccak256("admin");
        assert_eq!(admin.encode_hex::<String>(), ADMIN);
    }

    #[test]
    fn ecrecover_works() {
        let message = "Casper Message:\nAhoj przygodo!";
        let signature_hex = "1e87e186238fa1df9c222b387a79910388c6ef56285924c7e4f6d7e77ed1d6c61815312cf66a5318db204c693b79e020b1d392dafe8c1b3841e1f6b4c41ca0fa";
        let my_public_key_hex =
            "036d9b880e44254afaf34330e57703a63aec53b5918d4470059b67a4a906350105";

        let signature: [u8; 64] = hex::decode(signature_hex).unwrap().try_into().unwrap();
        let my_public_key: [u8; 33] = hex::decode(my_public_key_hex).unwrap().try_into().unwrap();
        let hash = sha2_256(message.as_bytes());

        let recovered_public_key = ecrecover(&hash, &signature, 0).unwrap();

        assert_eq!(recovered_public_key.serialize_compressed(), my_public_key);
    }

    #[test]
    fn signature_verification_works() {
        let message = "Message to be signed";
        let message_bytes = &Bytes::from(message.as_bytes().to_vec());
        let account = test_env::get_account(0);

        let signature = test_env::sign_message(message_bytes, &account);

        let public_key = test_env::public_key(&account);
        assert!(contract_env::verify_signature(message_bytes, &signature, &public_key));
    }
}
