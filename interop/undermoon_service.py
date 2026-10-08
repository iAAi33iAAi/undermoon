"""Undermoon AETHEL Interop v1 compatibility service.

The repository currently contains a legacy floating-point validator under
aethel-grid/rust-validator. The canonical SPEC-004 engine is not promoted
here until its golden-vector semantics and executable implementation agree.
This service exposes that distinction to the AETHEL control plane.
"""
from __future__ import annotations

import argparse
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any


PROTOCOL = "aethel-interop/1"
SERVICE = "undermoon"
VERSION = "compat-0.1.0"
LEGACY_VALIDATOR_PATH = Path(__file__).parents[1] / "aethel-grid" / "rust-validator"


def evaluate(request_id: str, operation: str, payload: dict[str, Any]) -> dict[str, Any]:
    if operation == "capabilities":
        return {
            "protocol": PROTOCOL,
            "service": SERVICE,
            "version": VERSION,
            "request_id": request_id,
            "status": "PASS",
            "decision": "DESCRIBED",
            "reasons": [],
            "result": {
                "legacy_validator": LEGACY_VALIDATOR_PATH.exists(),
                "canonical_spec": "SPEC-004",
            },
            "evidence": {
                "legacy_validator": str(LEGACY_VALIDATOR_PATH),
                "canonical_conformance": "blocked",
            },
        }

    if operation == "conformance-status":
        return {
            "protocol": PROTOCOL,
            "service": SERVICE,
            "version": VERSION,
            "request_id": request_id,
            "status": "BLOCKED",
            "decision": "HOLD",
            "reasons": [
                "Canonical SPEC-004 execution must not be inferred from the legacy floating-point validator.",
                "Resolve the authoritative golden-vector semantic mismatch before promotion.",
            ],
            "result": {
                "legacy_validator": "present",
                "canonical_validator": "not-promoted",
            },
            "evidence": {
                "source": "m4/spec004-v03-rigor-conformance",
                "policy": "no expected-hash edits to force a green conformance result",
            },
        }

    if operation == "validate-input":
        state = payload.get("state")
        if not isinstance(state, dict):
            return {
                "protocol": PROTOCOL, "service": SERVICE, "version": VERSION,
                "request_id": request_id, "status": "FAIL",
                "decision": "INVALID_INPUT",
                "reasons": ["state must be an object"],
                "result": {}, "evidence": {},
            }
        required = ("liquidity", "latency", "entropy", "eco_score", "ai_score", "phi_total")
        missing = [key for key in required if key not in state]
        if missing:
            return {
                "protocol": PROTOCOL, "service": SERVICE, "version": VERSION,
                "request_id": request_id, "status": "FAIL",
                "decision": "INVALID_INPUT",
                "reasons": [f"missing state fields: {missing}"],
                "result": {}, "evidence": {},
            }
        return {
            "protocol": PROTOCOL, "service": SERVICE, "version": VERSION,
            "request_id": request_id, "status": "PASS",
            "decision": "SCHEMA_VALID",
            "reasons": [],
            "result": {"fields": list(required)},
            "evidence": {"validator": VERSION},
        }

    return {
        "protocol": PROTOCOL, "service": SERVICE, "version": VERSION,
        "request_id": request_id, "status": "FAIL",
        "decision": "INVALID_OPERATION",
        "reasons": [f"unsupported operation: {operation}"],
        "result": {}, "evidence": {},
    }


class Handler(BaseHTTPRequestHandler):
    def _send(self, code: int, body: dict[str, Any]) -> None:
        raw = json.dumps(body, sort_keys=True).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_GET(self) -> None:
        if self.path == "/aethel/health":
            self._send(200, evaluate("health", "capabilities", {}))
        elif self.path == "/aethel/capabilities":
            self._send(200, evaluate("capabilities", "capabilities", {}))
        else:
            self._send(404, {"error": "not found"})

    def do_POST(self) -> None:
        if self.path != "/aethel/evaluate":
            self._send(404, {"error": "not found"})
            return
        try:
            size = int(self.headers.get("Content-Length", "0"))
            body = json.loads(self.rfile.read(size).decode("utf-8"))
            if body.get("protocol") != PROTOCOL:
                self._send(400, {"error": "unsupported protocol"})
                return
            self._send(
                200,
                evaluate(
                    str(body["request_id"]),
                    str(body.get("operation", "")),
                    dict(body.get("payload", {})),
                ),
            )
        except (TypeError, ValueError, KeyError, json.JSONDecodeError) as exc:
            self._send(400, {"error": str(exc)})

    def log_message(self, format: str, *args: Any) -> None:
        return


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8104)
    args = parser.parse_args()
    ThreadingHTTPServer((args.host, args.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
