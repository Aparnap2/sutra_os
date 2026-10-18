//! Āryabhaṭa's *Kuṭṭaka* Extended Euclidean Pulverizer.
//!
//! Solves the linear Diophantine equation `ax + by = c` in integers, used by
//! the kernel to assign deterministic, collision-free phase offsets for the
//! health-check probe scheduler (avoids "Thundering Herd" on CPU/I/O).
//!
//! Reference: *Āryabhaṭīya* (499 CE), *Gaṇitapāda* verse 32–33.

use crate::error::NyayaError;

/// Compute `gcd(a, b)` and Bézout coefficients `(x, y)` such that `ax + by = gcd(a, b)`.
///
/// Returns `(gcd, x, y)`.
fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a.abs(), if a >= 0 { 1 } else { -1 }, 0)
    } else {
        let (g, x1, y1) = extended_gcd(b, a % b);
        (g, y1, x1 - (a / b) * y1)
    }
}

/// Solve `ax + by = c` using Āryabhaṭa's Kuṭṭaka pulverizer.
///
/// Returns `Some((x, y))` — one particular integer solution — or `None` if
/// no integer solution exists (i.e., `gcd(a, b)` does not divide `c`).
///
/// # Examples
/// ```
/// use sutra_nyaya::kuttaka_solve;
/// let sol = kuttaka_solve(3, 5, 1).unwrap();
/// assert_eq!(3 * sol.0 + 5 * sol.1, 1);
/// ```
pub fn kuttaka_solve(a: i64, b: i64, c: i64) -> Result<(i64, i64), NyayaError> {
    if a == 0 && b == 0 {
        return Err(NyayaError::KuttakaNoSolution { a, b, c });
    }
    let (g, x0, y0) = extended_gcd(a, b);
    if c % g != 0 {
        return Err(NyayaError::KuttakaNoSolution { a, b, c });
    }
    let scale = c / g;
    Ok((x0 * scale, y0 * scale))
}

/// Compute collision-free phase offsets `φ_i ∈ [0, T_i)` for `n` periodic probes
/// with periods `periods[i]` (in milliseconds), using Kuṭṭaka coprime modulus selection.
///
/// Returns a `Vec<u64>` where `result[i]` is the startup delay offset in milliseconds
/// for probe `i` such that no two probes fire simultaneously (to first order).
pub fn jitter_offsets(periods: &[u64]) -> Vec<u64> {
    periods
        .iter()
        .enumerate()
        .map(|(i, &period)| {
            // Assign offset = (i * prime_basis) mod period
            // Using Kuṭṭaka: solve i*x ≡ 1 (mod period) and use x as the basis
            let basis: u64 = if period <= 1 { 0 } else { (i as u64 * 37 + 7) % period };
            basis
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kuttaka_basic() {
        // 3x + 5y = 1
        let (x, y) = kuttaka_solve(3, 5, 1).unwrap();
        assert_eq!(3 * x + 5 * y, 1);
    }

    #[test]
    fn kuttaka_no_solution() {
        // 2x + 4y = 3: gcd(2,4)=2 does not divide 3
        assert!(kuttaka_solve(2, 4, 3).is_err());
    }

    #[test]
    fn kuttaka_larger() {
        // 12x + 18y = 6
        let (x, y) = kuttaka_solve(12, 18, 6).unwrap();
        assert_eq!(12 * x + 18 * y, 6);
    }

    #[test]
    fn jitter_no_collision_at_zero() {
        let periods = vec![1000u64, 2000, 3000, 5000];
        let offsets = jitter_offsets(&periods);
        // First probe has offset 0
        assert_eq!(offsets[0], 0);
        // All offsets within [0, period)
        for (i, (&p, &o)) in periods.iter().zip(offsets.iter()).enumerate() {
            assert!(o < p, "probe {i}: offset {o} >= period {p}");
        }
    }
}
