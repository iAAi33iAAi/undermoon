# M4 — SPEC-004 v0.3_RIGOR Golden Conformance

This directory makes the M4 golden corpus the authoritative byte-level replay target for the Rust side.

## What is verified

For each immutable vector, the test:

1. Deserializes the structured record.
2. Re-serializes it with serde_json in the declared struct field order.
3. Requires the resulting UTF-8 bytes to equal the supplied audit_preimage exactly.
4. Computes SHA-256 over those exact bytes.
5. Requires the digest to equal the supplied expected_sha256.
6. Preserves the exact integer result_state, u_metric, and decision values from the corpus.

The corpus contains integer-only numeric fields. No floating-point value is admitted into the audit hash domain.

## Important boundary

This now verifies raw-input -> result_state for the four supplied vectors, including the frozen rotation law and integer truncation. It remains a golden gate rather than full Level-4 proof because the exact normative u_metric equation and complete decision partition are still not present in the available specification. The normative Rust execution engine must supply those next; its generated metric, decision, preimage bytes, and digest must match the corpus exactly. A hard mismatch is a failure.

## Run

cargo test --manifest-path aethel-grid/m4-conformance/Cargo.toml
cargo run --manifest-path aethel-grid/m4-conformance/Cargo.toml

Expected result: 4/4 vectors PASS.
