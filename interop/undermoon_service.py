"""Undermoon AETHEL Interop v1 compatibility service.

The repository currently contains a legacy floating-point validator under
aethel-grid/rust-validator. Canonical SPEC-004 conformance is intentionally
not promoted until the authoritative golden-vector semantics are reconciled.
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

def _response(request_id: str, status: str, decision: str, reasons: list[str], result: dict[str, Any], evidence: dict[str, Any]) -> dict[str, Any]:
    return {"protocol": PROTOCOL, "service": SERVICE, "version": VERSION, "request_id": request_id, "status": status, "decision": decision, "reasons": reasons, "result": result, "evidence": evidence}

def evaluate(request_id: str, operation: str, payload: dict[str, Any]) -> dict[str, Any]:
    if operation == "capabilities":
        return _response(request_id, "PASS", "DESCRIBED", [], {"legacy_validator": LEGACY_VALIDATOR_PATH.exists(), "canonical_spec": "SPEC-004", "operations": ["capabilities", "conformance-status", "validate-input"]}, {"canonical_conformance": "blocked"})
    if operation == "conformance-status":
        return _response(request_id, "BLOCKED", "HOLD", ["Canonical SPEC-004 execution must not be inferred from the legacy floating-point validator.", "Resolve the authoritative golden-vector semantic mismatch before promotion."], {"legacy_validator": "present" if LEGACY_VALIDATOR_PATH.exists() else "missing", "canonical_validator": "not-promoted"}, {"policy": "do not edit expected hashes to force a green conformance result"})
    if operation == "validate-input":
        state = payload.get("state")
        if not isinstance(state, dict):
            return _response(request_id, "FAIL", "INVALID_INPUT", ["state must be an object"], {}, {})
        required = ("liquidity", "latency", "entropy", "eco_score", "ai_score", "phi_total")
        missing = [key for key in required if key not in state]
        if missing:
            return _response(request_id, "FAIL", "INVALID_INPUT", [f"missing state fields: {missing}"], {}, {})
        return _response(request_id, "PASS", "SCHEMA_VALID", [], {"fields": list(required)}, {"validator": VERSION})
    return _response(request_id, "FAIL", "INVALID_OPERATION", [f"unsupported operation: {operation}"], {}, {})

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
            self._send(200, _response("health", "PASS", "HEALTHY", [], {"legacy_validator_present": LEGACY_VALIDATOR_PATH.exists()}, {"canonical_conformance": "blocked"}))
            return
        if self.path == "/aethel/capabilities":
            self._send(200, evaluate("capabilities", "capabilities", {}))
            return
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
            self._send(200, evaluate(str(body["request_id"]), str(body.get("operation", "")), dict(body.get("payload", {}))))
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
