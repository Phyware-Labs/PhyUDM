//! The verifier against the conformance vectors every implementation shares.

use serde_json::Value;
use udm_verify::canonical::{leaf_payload, signing_payload};
use udm_verify::merkle::{audit_path, leaf_hash, merkle_root, root_from_proof};
use udm_verify::{hex, KeyMaterial, Keyring, SignatureMode, SignatureStatus, Verifier};

fn vectors(name: &str) -> Value {
    let path = format!(
        "{}/../../conformance/vectors/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn signing_payloads_and_hmacs() {
    let doc = vectors("provenance-signing.json");
    let list = doc["vectors"].as_array().unwrap();
    assert_ne!(list.len(), 0);
    for v in list {
        let event = &v["event"];
        assert_eq!(
            signing_payload(event),
            v["payload"].as_str().unwrap(),
            "{}",
            v["name"]
        );
        // The reference HMAC is over the payload, whatever algorithm the
        // event names.
        let key = KeyMaterial::Hmac(v["key"].as_str().unwrap().as_bytes().to_vec());
        assert!(
            key.verify(
                signing_payload(event).as_bytes(),
                v["hmac_sha256"].as_str().unwrap()
            ),
            "{}",
            v["name"]
        );
    }
}

#[test]
fn merkle_trees() {
    for tree in vectors("merkle-batch.json")["trees"].as_array().unwrap() {
        let leaves: Vec<hex::Hash> = tree["payloads"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| leaf_hash(p.as_str().unwrap().as_bytes()))
            .collect();
        assert_eq!(hex::encode(&merkle_root(&leaves)), tree["root"]);
        let size = leaves.len() as u64;
        for (i, proof) in tree["proofs"].as_array().unwrap().iter().enumerate() {
            let proof: Vec<hex::Hash> = proof
                .as_array()
                .unwrap()
                .iter()
                .map(|h| hex::hash(h.as_str().unwrap()).unwrap())
                .collect();
            assert_eq!(audit_path(i, &leaves).unwrap(), proof);
            assert_eq!(
                root_from_proof(i as u64, size, &leaves[i], &proof),
                Some(merkle_root(&leaves))
            );
        }
    }
}

#[test]
fn batch_signed_events() {
    let batch = &vectors("merkle-batch.json")["batch"];
    let events = batch["events"].as_array().unwrap();
    let key_id = events[0]["provenance"]["key_id"].as_str().unwrap();
    let mut keyring = Keyring::new();
    keyring.add_hmac(key_id, batch["key"].as_str().unwrap().as_bytes());
    let mut verifier = Verifier::new(&keyring);
    for (event, payload) in events
        .iter()
        .zip(batch["leaf_payloads"].as_array().unwrap())
    {
        assert_eq!(&leaf_payload(event), payload);
        let check = verifier.check_event(event);
        assert_eq!(check.status, SignatureStatus::Verified);
        assert_eq!(check.mode, Some(SignatureMode::Batch));
        let mut forged = event.clone();
        forged["source_id"] = "someone-else".into();
        assert_eq!(
            verifier.check_event(&forged).status,
            SignatureStatus::InvalidSignature
        );
    }
}

#[test]
fn unknown_key_and_unsigned() {
    let doc = vectors("merkle-batch.json");
    let event = &doc["batch"]["events"][0];
    let empty = Keyring::new();
    assert_eq!(
        Verifier::new(&empty).check_event(event).status,
        SignatureStatus::UnknownKey
    );
    let mut bare = event.clone();
    bare.as_object_mut().unwrap().remove("provenance");
    assert_eq!(
        Verifier::new(&empty).check_event(&bare).status,
        SignatureStatus::Unsigned
    );
}
