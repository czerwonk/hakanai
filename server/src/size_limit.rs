// SPDX-License-Identifier: Apache-2.0

// Calculates the size limit accounting for the overhead of a secret
//
// The 1.5x factor accounts for:
// - Base64 encoding adds ~33% overhead (4 bytes output for every 3 bytes input)
// - AES-256-GCM encryption adds 28 bytes (12 byte nonce + 16 byte auth tag)
// - JSON structure overhead for the API request
// - Small buffer for field names and formatting
//
// For typical payloads, 1.5x provides sufficient headroom while preventing
// excessively large uploads that could cause memory issues.
pub fn calculate(configured_limit: usize) -> usize {
    // use factor 1.5 to account for overhead in base64 encoding and encryption
    configured_limit.saturating_mul(3).saturating_div(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_typical_sizes() {
        // Test common size limits
        assert_eq!(calculate(1024), 1536, "1KB should calculate to 1.5KB"); // 1KB -> 1.5KB
        assert_eq!(
            calculate(1_048_576),
            1_572_864,
            "1MB should calculate to 1.5MB"
        ); // 1MB -> 1.5MB
        assert_eq!(
            calculate(10_485_760),
            15_728_640,
            "10MB should calculate to 15MB"
        ); // 10MB -> 15MB
        assert_eq!(
            calculate(104_857_600),
            157_286_400,
            "100MB should calculate to 150MB"
        ); // 100MB -> 150MB
    }

    #[test]
    fn test_calculate_small_sizes() {
        assert_eq!(calculate(0), 0, "Zero input should return zero");
        assert_eq!(
            calculate(1),
            1,
            "1 byte should round down to 1 with integer division"
        ); // 1 * 3 / 2 = 1 (integer division)
        assert_eq!(calculate(2), 3, "2 bytes should calculate to 3"); // 2 * 3 / 2 = 3
        assert_eq!(
            calculate(3),
            4,
            "3 bytes should round down to 4 with integer division"
        ); // 3 * 3 / 2 = 4 (integer division)
        assert_eq!(calculate(100), 150, "100 bytes should calculate to 150");
    }

    #[test]
    fn test_calculate_odd_sizes() {
        // Test that odd sizes don't cause issues with integer division
        assert_eq!(
            calculate(333),
            499,
            "Odd number 333 should calculate correctly with integer division"
        ); // 333 * 3 / 2 = 499
        assert_eq!(
            calculate(1001),
            1501,
            "Odd number 1001 should calculate correctly"
        ); // 1001 * 3 / 2 = 1501
        assert_eq!(
            calculate(9999),
            14998,
            "Odd number 9999 should calculate correctly"
        ); // 9999 * 3 / 2 = 14998
    }
}
