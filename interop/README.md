# Undermoon AETHEL Interop

This branch adds an AETHEL Interop v1 compatibility surface.

The service deliberately distinguishes the existing legacy floating-point
validator from canonical SPEC-004 conformance. Canonical conformance remains
blocked until the authoritative vectors, canonical audit preimage, and Rust
execution semantics agree.

Run:

    python -m interop.undermoon_service --host 127.0.0.1 --port 8104

Endpoints:

    GET  /aethel/health
    GET  /aethel/capabilities
    POST /aethel/evaluate

