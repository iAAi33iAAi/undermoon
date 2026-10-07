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

This is a golden-replay gate, not a proof that an arbitrary Rust implementation derives the four outputs from raw inputs. The next Level-4 step is to run the same corpus through the normative Rust execution engine and compare its generated state, metric, decision, preimage bytes, and digest against this corpus. A hard mismatch is a failure.

## Run

cargo test --manifest-path aethel-grid/m4-conformance/Cargo.toml
cargo run --manifest-path aethel-grid/m4-conformance/Cargo.toml

Expected result: 4/4 vectors PASS.
