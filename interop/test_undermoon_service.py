from interop.undermoon_service import evaluate


def test_conformance_is_explicitly_blocked():
    result = evaluate("r1", "conformance-status", {})
    assert result["status"] == "BLOCKED"
    assert result["decision"] == "HOLD"


def test_state_schema_contract():
    state = {
        "liquidity": 0.9,
        "latency": 0.9,
        "entropy": 0.9,
        "eco_score": 0.9,
        "ai_score": 0.9,
        "phi_total": 0.9,
    }
    result = evaluate("r2", "validate-input", {"state": state})
    assert result["status"] == "PASS"
    assert result["decision"] == "SCHEMA_VALID"
