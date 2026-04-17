// src/lib.rs - Production FFI + Consensus
use std::os::raw::{c_double, c_int};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockState {
    pub liquidity: f64,   // P301 domain
    pub latency: f64,     // Network health
    pub entropy: f64,     // P1 thermal
    pub eco_score: f64,   // Phi_total human weight
    pub ai_score: f64,    // Phi_total AI weight
    pub phi_total: f64,   // Master stability signal
}

extern "C" {
    fn p1_thermal_equilibrium(t_field: *const c_double, len: c_int) -> c_double;
    fn p701_sinkhorn_sns(cost_matrix: *const c_double, len: c_int) -> c_double;
}

pub fn verify_stability(state: &BlockState) -> bool {
    unsafe {
        let data: [f64; 6] = [
            state.liquidity, state.latency, state.entropy,
            state.eco_score, state.ai_score, state.phi_total
        ];
        let thermal = p1_thermal_equilibrium(data.as_ptr(), 6);
        let sync = p701_sinkhorn_sns(data.as_ptr(), 6);
        // Physics gates: 85% stability floor
        thermal >= 0.85 && sync >= 0.85 && state.phi_total >= 0.90
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stability_pass() {
        let state = BlockState {
            liquidity: 0.95, latency: 0.92, entropy: 0.88,
            eco_score: 0.91, ai_score: 0.89, phi_total: 0.92
        };
        assert!(verify_stability(&state));
    }
}
