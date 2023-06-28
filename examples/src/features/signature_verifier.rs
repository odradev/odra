use odra::contract_env;
use odra::types::{Bytes, PublicKey};

#[odra::module]
pub struct SignatureVerifier {}

#[odra::module]
impl SignatureVerifier {
    pub fn verify_signature(
        &self,
        message: &Bytes,
        signature: &Bytes,
        public_key: &PublicKey
    ) -> bool {
        contract_env::verify_signature(message, signature, public_key)
    }
}

#[cfg(test)]
mod test {
    use crate::features::signature_verifier::SignatureVerifierDeployer;
    use odra::test_env;
    use odra::types::Bytes;

    #[test]
    fn signature_verification_works() {
        let message = "Message to be signed";
        let message_bytes = &Bytes::from(message.as_bytes().to_vec());
        let account = test_env::get_account(0);

        let signature = test_env::sign_message(message_bytes, &account);

        let public_key = test_env::public_key(&account);

        let signature_verifier = SignatureVerifierDeployer::default();
        assert!(signature_verifier.verify_signature(message_bytes, &signature, &public_key));
    }

    /// The following test checks that the signature verification works with the signature produced
    /// by the casper wallet.
    #[test]
    #[cfg(feature = "casper")]
    fn verify_signature_casper_wallet() {
        use odra::casper::casper_types::bytesrepr::FromBytes;
        // Casper Wallet for the message "Ahoj przygodo!" signed using SECP256K1 key
        // produces the following signature:
        // 1e87e186238fa1df9c222b387a79910388c6ef56285924c7e4f6d7e77ed1d6c61815312cf66a5318db204c693b79e020b1d392dafe8c1b3841e1f6b4c41ca0fa
        // Casper Wallet adds "Casper Message:\n" prefix to the message:
        let message = "Casper Message:\nAhoj przygodo!";
        let message_bytes = &Bytes::from(message.as_bytes().to_vec());

        // Depending on the type of the key, we need to prefix the signature with a tag:
        // 0x01 for ED25519
        // 0x02 for SECP256K1
        let signature_hex = "021e87e186238fa1df9c222b387a79910388c6ef56285924c7e4f6d7e77ed1d6c61815312cf66a5318db204c693b79e020b1d392dafe8c1b3841e1f6b4c41ca0fa";
        let signature: [u8; 65] = hex::decode(signature_hex).unwrap().try_into().unwrap();
        let signature_bytes = &Bytes::from(signature.to_vec());

        // Similar to the above, the public key is tagged:
        let public_key_hex = "02036d9b880e44254afaf34330e57703a63aec53b5918d4470059b67a4a906350105";
        let public_key_decoded = hex::decode(public_key_hex).unwrap();
        let (public_key, _) = odra::casper::casper_types::crypto::PublicKey::from_bytes(
            public_key_decoded.as_slice()
        )
        .unwrap();

        let signature_verifier = SignatureVerifierDeployer::default();
        assert!(signature_verifier.verify_signature(message_bytes, signature_bytes, &public_key));
    }
}
