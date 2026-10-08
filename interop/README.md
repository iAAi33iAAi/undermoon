# Undermoon AETHEL Interop

This branch exposes a versioned aethel-interop/1 compatibility surface.

The service explicitly separates legacy floating-point validation code from
canonical SPEC-004 conformance.

Canonical conformance remains blocked until the authoritative golden vectors
and canonical audit preimage semantics agree. The bridge never changes the
expected vectors to make the conformance gate pass.

Run:
    python -m interop.undermoon_service --host 127.0.0.1 --port 8104

Endpoints:
    GET  /aethel/health
    GET  /aethel/capabilities
    POST /aethel/evaluate
