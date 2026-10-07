use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const CORPUS_PATH: &str = "../golden_corpus.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct InputBundle {
    evidence: [i64; 6],
    density: [i64; 6],
    source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct AuditRecord {
    spec_version: String,
    prev_state_hash: String,
    timestamp: String,
    input_bundle: InputBundle,
    operator: String,
    operator_version: String,
    applied_transforms: Vec<String>,
    result_state: [i64; 12],
    u_metric: i64,
    decision: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GoldenVector {
    id: String,
    audit_record: AuditRecord,
    audit_preimage: String,
    expected_sha256: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn load_corpus() -> Vec<GoldenVector> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_PATH);
    let raw = fs::read_to_string(path).expect("M4 golden corpus missing");
    serde_json::from_str(&raw).expect("M4 golden corpus is not valid JSON")
}

fn assert_vector(v: &GoldenVector) {
    let canonical = serde_json::to_string(&v.audit_record)
        .expect("AuditRecord serialization failed");

    assert_eq!(
        canonical.as_bytes(),
        v.audit_preimage.as_bytes(),
        "{}: canonical audit preimage mismatch",
        v.id
    );

    let digest = sha256_hex(canonical.as_bytes());
    assert_eq!(
        digest, v.expected_sha256,
        "{}: SHA-256 mismatch: got {}, expected {}",
        v.id, digest, v.expected_sha256
    );

    assert!(
        !v.audit_preimage.as_bytes().contains(&b'\n'),
        "{}: preimage contains newline",
        v.id
    );
    assert!(
        !v.audit_preimage.as_bytes().contains(&b'\r'),
        "{}: preimage contains carriage return",
        v.id
    );

    println!(
        "{} PASS sha256={} u_metric={} decision={} state={:?}",
        v.id, digest, v.audit_record.u_metric, v.audit_record.decision, v.audit_record.result_state
    );
}

#[test]
fn m4_all_golden_vectors_are_bit_exact() {
    let corpus = load_corpus();
    assert_eq!(corpus.len(), 4, "M4 corpus must contain exactly four vectors");
    for v in &corpus {
        assert_vector(v);
    }
}

fn main() {
    let corpus = load_corpus();
    assert_eq!(corpus.len(), 4, "M4 corpus must contain exactly four vectors");
    for v in &corpus {
        assert_vector(v);
    }
}
