use alloy::{
    primitives::{Address, Signature, B256, U256},
    signers::{local::PrivateKeySigner, SignerSync},
    sol,
    sol_types::{eip712_domain, SolStruct},
};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferWithAuthorization {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub valid_after: U256,
    pub valid_before: U256,
    pub nonce: B256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentSignature {
    pub v: u8,
    pub r: B256,
    pub s: B256,
}

impl PaymentSignature {
    pub fn to_bytes(&self) -> [u8; 65] {
        let mut bytes = [0u8; 65];
        bytes[..32].copy_from_slice(self.r.as_slice());
        bytes[32..64].copy_from_slice(self.s.as_slice());
        bytes[64] = self.v;
        bytes
    }
}

#[derive(Error, Debug)]
pub enum SigningError {
    #[error("invalid EIP-712 domain: {0}")]
    Domain(String),
    #[error("invalid transfer authorization: {0}")]
    Authorization(String),
    #[error("signature failed: {0}")]
    Signer(String),
}

pub type Result<T> = std::result::Result<T, SigningError>;

pub fn sign_transfer_with_authorization(
    authorization: TransferWithAuthorization,
    chain_id: u64,
    verifying_contract: Address,
    signer: &PrivateKeySigner,
) -> Result<PaymentSignature> {
    let digest = transfer_with_authorization_digest(&authorization, chain_id, verifying_contract)?;
    let signature = signer
        .sign_hash_sync(&digest)
        .map_err(|err| SigningError::Signer(err.to_string()))?;

    Ok(payment_signature_from_alloy(signature))
}

pub fn transfer_with_authorization_digest(
    authorization: &TransferWithAuthorization,
    chain_id: u64,
    verifying_contract: Address,
) -> Result<B256> {
    validate_authorization(authorization)?;
    validate_domain(chain_id, verifying_contract)?;

    let domain = eip712_domain! {
        name: "USD Coin",
        version: "2",
        chain_id: chain_id,
        verifying_contract: verifying_contract,
    };
    let typed = typed_data::TransferWithAuthorization {
        from: authorization.from,
        to: authorization.to,
        value: authorization.value,
        validAfter: authorization.valid_after,
        validBefore: authorization.valid_before,
        nonce: authorization.nonce,
    };

    Ok(typed.eip712_signing_hash(&domain))
}

fn validate_domain(chain_id: u64, verifying_contract: Address) -> Result<()> {
    if chain_id == 0 {
        return Err(SigningError::Domain("chain_id must be non-zero".into()));
    }
    if verifying_contract.is_zero() {
        return Err(SigningError::Domain(
            "verifying contract must be non-zero".into(),
        ));
    }
    Ok(())
}

fn validate_authorization(authorization: &TransferWithAuthorization) -> Result<()> {
    if authorization.from.is_zero() {
        return Err(SigningError::Authorization("from must be non-zero".into()));
    }
    if authorization.to.is_zero() {
        return Err(SigningError::Authorization("to must be non-zero".into()));
    }
    if authorization.valid_before <= authorization.valid_after {
        return Err(SigningError::Authorization(
            "valid_before must be greater than valid_after".into(),
        ));
    }
    Ok(())
}

fn payment_signature_from_alloy(signature: Signature) -> PaymentSignature {
    PaymentSignature {
        v: 27 + u8::from(signature.v()),
        r: B256::from(signature.r().to_be_bytes::<32>()),
        s: B256::from(signature.s().to_be_bytes::<32>()),
    }
}

mod typed_data {
    use super::*;

    sol! {
        struct TransferWithAuthorization {
            address from;
            address to;
            uint256 value;
            uint256 validAfter;
            uint256 validBefore;
            bytes32 nonce;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    const CHAIN_ID: u64 = 84532;
    const EXPECTED_TYPE: &str = "TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)";

    fn signer() -> PrivateKeySigner {
        "0x59c6995e998f97a5a0044966f0945389d358f57d07535c8be9e515a7c99316c5"
            .parse()
            .unwrap()
    }

    fn verifying_contract() -> Address {
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    }

    fn authorization() -> TransferWithAuthorization {
        TransferWithAuthorization {
            from: signer().address(),
            to: address!("1111111111111111111111111111111111111111"),
            value: U256::from(50000),
            valid_after: U256::from(1_700_000_000u64),
            valid_before: U256::from(1_700_003_600u64),
            nonce: B256::from([7u8; 32]),
        }
    }

    #[test]
    fn transfer_typehash_matches_usdc_v2() {
        assert_eq!(
            typed_data::TransferWithAuthorization::eip712_encode_type(),
            EXPECTED_TYPE
        );
    }

    #[test]
    fn signature_is_deterministic_for_same_nonce() {
        let signature_a = sign_transfer_with_authorization(
            authorization(),
            CHAIN_ID,
            verifying_contract(),
            &signer(),
        )
        .unwrap();
        let signature_b = sign_transfer_with_authorization(
            authorization(),
            CHAIN_ID,
            verifying_contract(),
            &signer(),
        )
        .unwrap();

        assert_eq!(signature_a, signature_b);
        assert_eq!(signature_a.to_bytes().len(), 65);
    }

    #[test]
    fn recovered_signature_matches_signer() {
        let signer = signer();
        let digest =
            transfer_with_authorization_digest(&authorization(), CHAIN_ID, verifying_contract())
                .unwrap();
        let signature = sign_transfer_with_authorization(
            authorization(),
            CHAIN_ID,
            verifying_contract(),
            &signer,
        )
        .unwrap();
        let alloy_signature = Signature::from_raw(&signature.to_bytes()).unwrap();

        assert_eq!(
            alloy_signature
                .recover_address_from_prehash(&digest)
                .unwrap(),
            signer.address()
        );
    }

    #[test]
    fn exact_fixture_matches_expected_signature_bytes() {
        let signature = sign_transfer_with_authorization(
            authorization(),
            CHAIN_ID,
            verifying_contract(),
            &signer(),
        )
        .unwrap();

        assert_eq!(
            signature.to_bytes(),
            hex_to_65("50352ae33b921be27b86127d2f156470caf5fa5ba371380d995e6c66c58d374f411b4483ca2f95ac5cc8dc3ccf36e4171d8be737585b856a748f10b616416e3e1c")
        );
    }

    #[test]
    fn rejects_invalid_domain() {
        let err = transfer_with_authorization_digest(&authorization(), 0, verifying_contract())
            .unwrap_err();

        assert!(matches!(err, SigningError::Domain(_)));
    }

    #[test]
    fn rejects_invalid_authorization() {
        let err = transfer_with_authorization_digest(
            &TransferWithAuthorization {
                valid_before: U256::from(1),
                valid_after: U256::from(1),
                ..authorization()
            },
            CHAIN_ID,
            verifying_contract(),
        )
        .unwrap_err();

        assert!(matches!(err, SigningError::Authorization(_)));
    }

    fn hex_to_65(value: &str) -> [u8; 65] {
        let value = value.strip_prefix("0x").unwrap_or(value);
        let mut bytes = [0u8; 65];
        hex::decode_to_slice(value, &mut bytes).unwrap();
        bytes
    }
}
