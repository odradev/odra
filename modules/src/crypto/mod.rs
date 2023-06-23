use sha3::{Digest, Keccak256};

pub(crate) fn keccak256(input: &str) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let result = result.as_slice();
    result.try_into().unwrap()
}

#[cfg(test)]
mod test {
    use super::keccak256;
    use hex::ToHex;
    use odra::{contract_env, test_env};
    use odra::types::Bytes;

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
    fn signature_verification_works() {
        let message = "Message to be signed";
        let message_bytes = &Bytes::from(message.as_bytes().to_vec());
        let account = test_env::get_account(0);

        let signature = test_env::sign_message(message_bytes, &account);

        let public_key = test_env::public_key(&account);
        assert!(contract_env::verify_signature(message_bytes, &signature, &public_key));
    }
}
