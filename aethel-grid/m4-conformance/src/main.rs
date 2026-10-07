use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const CORPUS_PATH: &str = "../golden_corpus.json";

const Q: i128 = 1_000_000;
const I64_MIN: i128 = i64::MIN as i128;
const I64_MAX: i128 = i64::MAX as i128;

/// Frozen SPEC-004 v0.3_RIGOR integer rotation coefficients.
fn rotation_coefficients(angle_deg: i32) -> (i128, i128) {
    match angle_deg {
        0 => (1_000_000, 0),
        15 => (965_926, 258_819),
        30 => (866_025, 500_000),
        45 => (707_107, 707_107),
        60 => (500_000, 866_025),
        90 => (0, 1_000_000),
        180 => (-1_000_000, 0),
        270 => (0, -1_000_000),
        _ => panic!("unsupported normative rotation angle: {angle_deg}"),
    }
}

/// Rust signed division is truncation toward zero, matching SPEC-004 qdiv.
fn qdiv(numerator: i128) -> i64 {
    let value = numerator / Q;
    assert!(
        (I64_MIN..=I64_MAX).contains(&value),
        "qdiv result outside i64 range: {value}"
    );
    value as i64
}

/// Derive the 12-element fixed-point result state from the raw vector inputs.
///
/// State layout is [density_0, evidence_0, density_1, evidence_1, ...].
/// A rotate_d1_<angle> transform applies only to Domain 1's pair
/// [density_1, evidence_1]. All multiplication is promoted to i128 before
/// qdiv truncation back to i64.
fn derive_result_state(record: &AuditRecord) -> [i64; 12] {
    let mut out = [0_i64; 12];

    for domain in 0..6 {
        out[domain * 2] = record.input_bundle.density[domain];
        out[domain * 2 + 1] = record.input_bundle.evidence[domain];
    }

    let angle = record.applied_transforms.as_slice();
    if angle == ["none"] || angle.is_empty() {
        return out;
    }

    assert_eq!(angle.len(), 1, "exactly one transform is expected");
    let name = &angle[0];
    let prefix = "rotate_d1_";
    let angle_text = name.strip_prefix(prefix)
        .unwrap_or_else(|| panic!("unsupported transform: {name}"));
    let angle_deg: i32 = angle_text.parse()
        .unwrap_or_else(|_| panic!("invalid rotation angle in transform: {name}"));

    let (c, s) = rotation_coefficients(angle_deg);
    let real = record.input_bundle.density[1] as i128;
    let imag = record.input_bundle.evidence[1] as i128;

    let real_num = c * real - s * imag;
    let imag_num = s * real + c * imag;

    out[2] = qdiv(real_num);
    out[3] = qdiv(imag_num);
    out
}


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
    let generated_state = derive_result_state(&v.audit_record);
    assert_eq!(
        generated_state,
        v.audit_record.result_state,
        "{}: raw_input -> result_state derivation mismatch",
        v.id
    );
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
        "{} PASS state=DERIVED sha256={} u_metric={} decision={} state={:?}",
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
