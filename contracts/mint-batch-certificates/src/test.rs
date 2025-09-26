#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Env, BytesN, Address, Vec};
use super::{CertificateContract, CertificateContractClient};
use crate::certificate::{CertificateData, CertificateType};

#[test]
fn test_basic_contract_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // Create admin, issuer, and owner addresses
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let owner = Address::generate(&env);
    let max_batch_size = 10u32;

    // Deploy contract
    #[allow(deprecated)]
    let contract_id = env.register_contract(None, CertificateContract);
    let client = CertificateContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &max_batch_size);

    // Add issuer
    env.mock_all_auths();
    client.add_issuer(&admin, &issuer);

    // Verify issuer was added
    assert!(client.is_issuer(&issuer));

    // Create certificate data
    let cert_id = 1u64;
    let metadata_hash = BytesN::from_array(&env, &[1; 32]);
    let certificate = CertificateData {
        id: cert_id,
        metadata_hash: metadata_hash.clone(),
        valid_from: env.ledger().timestamp(),
        valid_until: env.ledger().timestamp() + 86400,
        revocable: true,
        cert_type: CertificateType::Standard,
    };

    // Mint certificate
    env.mock_all_auths();
    client.mint_single_certificate(&issuer, &owner, &certificate);

    // Verify certificate exists
    let stored_cert = client.get_certificate(&cert_id);
    assert!(stored_cert.is_some());
    
    let stored_cert = stored_cert.unwrap();
    assert_eq!(stored_cert.id, cert_id);
    assert_eq!(stored_cert.metadata_hash, metadata_hash);
}

// #[test]
// fn test_dynamic_batch_size_and_gas_estimation() {
//     let env = Env::default();
//     env.mock_all_auths();
//     let admin = Address::generate(&env);
//     let issuer = Address::generate(&env);
//     let max_batch_size = 10u32;
//     let contract_id = env.register(super::CertificateContract, {});
//     let client = super::CertificateContractClient::new(&env, &contract_id);
//     client.initialize(&admin, &max_batch_size);
//     client.add_issuer(&admin, &issuer);
//     // Create a large batch
//     let batch_size = 25u32;
//     let mut owners = Vec::new(&env);
//     let mut certs = Vec::new(&env);
//     for i in 0..batch_size {
//         let owner = Address::generate(&env);
//         let cert = CertificateData {
//             id: i as u64,
//             metadata_hash: BytesN::from_array(&env, &[i as u8; 32]),
//             valid_from: env.ledger().timestamp(),
//             valid_until: env.ledger().timestamp() + 86400,
//             revocable: true,
//             cert_type: CertificateType::Standard,
//         };
//         owners.push_back(owner);
//         certs.push_back(cert);
//     }
//     // Simulate a target gas limit
//     let target_gas_limit = 60_000u64;
//     let (estimated_gas, optimal_size) = super::CertificateContract::estimate_gas_for_batch(
//         &env, issuer.clone(), owners.clone(), certs.clone(), target_gas_limit
//     );
//     // The optimal size should be less than or equal to batch_size
//     assert!(optimal_size <= batch_size);
//     // The estimated gas should not exceed the target limit for the optimal batch
//     assert!(estimated_gas <= target_gas_limit || optimal_size == batch_size);
//     // Test dynamic batch minting
//     let results = super::CertificateContract::mint_batch_certificates_dynamic(
//         &env, issuer, owners, certs, target_gas_limit
//     );
//     // All certificates should be processed
//     assert_eq!(results.len() as u32, batch_size);
// }

#[test]
fn test_error_recovery_and_logging() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let owner = Address::generate(&env);
    let max_batch_size = 10u32;
    let contract_id = env.register(super::CertificateContract, {});
    let client = super::CertificateContractClient::new(&env, &contract_id);
    client.initialize(&admin, &max_batch_size);
    client.add_issuer(&admin, &issuer);
    // Create a certificate that will trigger a storage error (simulate by using a duplicate ID)
    let cert_id = 42u64;
    let certificate = CertificateData {
        id: cert_id,
        metadata_hash: BytesN::from_array(&env, &[42; 32]),
        valid_from: env.ledger().timestamp(),
        valid_until: env.ledger().timestamp() + 86400,
        revocable: true,
        cert_type: CertificateType::Standard,
    };
    // Mint once (should succeed)
    env.mock_all_auths();
    let result1 = client.try_mint_single_certificate(&issuer, &owner, &certificate);
    assert!(result1.is_ok());
    // Mint again (should fail with duplicate, and log error)
    let result2 = client.try_mint_single_certificate(&issuer, &owner, &certificate);
    assert!(result2.is_err());
    // There should be an error event for duplicate certificate
    // (In a real test, you would check the event log; here, we assert the error type)
    if let Err(e) = result2 {
        assert_eq!(e, Ok(super::error::Error::DuplicateCertificate));
    }
}
