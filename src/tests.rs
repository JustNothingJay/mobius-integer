use super::*;
use num_bigint::BigInt;

// =========================================================================
// The Proof — i64::MAX + 1
// =========================================================================

#[test]
fn the_proof_max_plus_one() {
    let max = Mi::new(i64::MAX);
    let result = max + Mi::new(1);

    // Machine strand wrapped (this is what C/Java/Go do)
    assert_eq!(result.machine(), i64::MIN);

    // Exact strand knows the truth
    assert!(result.is_corrupted());
    assert_eq!(
        result.collapse(),
        BigInt::from(i64::MAX) + BigInt::from(1)
    );
}

// =========================================================================
// Ariane 5 — June 4, 1996
//
// A 64-bit float was narrowed to a 16-bit signed integer.
// The value exceeded 32,767. The conversion overflowed silently.
// The Inertial Reference System failed. The backup had the same code.
// The rocket self-destructed 37 seconds after launch.
// $370 million payload lost.
//
// With MöbiusInteger, the narrow_i16() call would have returned Err.
// =========================================================================

#[test]
fn ariane_5_horizontal_velocity() {
    // The actual value was approximately 32,768 — just past i16::MAX
    let velocity = Mi::new(32_768);

    // Old way: silent truncation to i16 would wrap to -32768 or crash
    // MöbiusInteger way: the exact strand catches it
    assert!(velocity.narrow_i16().is_err());

    let err = velocity.narrow_i16().unwrap_err();
    assert_eq!(err.target_bits, 16);

    // A value that fits is fine
    let safe_velocity = Mi::new(32_767);
    assert_eq!(safe_velocity.narrow_i16().unwrap(), 32_767);
}

#[test]
fn ariane_5_the_rocket_lives() {
    // Simulate the accumulation that caused the real failure
    let mut velocity = Mi::new(0);
    let increment = Mi::new(100);

    for _ in 0..400 {
        velocity = velocity + increment.clone();
    }

    // velocity = 40,000 — way past i16::MAX (32,767)
    assert_eq!(velocity.collapse(), BigInt::from(40_000));

    // The narrow catches it before it kills anything
    assert!(velocity.narrow_i16().is_err());

    // The exact strand never lost track
    assert!(!velocity.is_corrupted()); // still fits in i64, so machine == exact
}

// =========================================================================
// YouTube / Gangnam Style — December 2014
//
// PSY's "Gangnam Style" exceeded 2,147,483,647 views (i32::MAX).
// YouTube's view counter was a 32-bit signed integer.
// Google had to upgrade to i64.
//
// With MöbiusInteger, the narrow_i32() call would have caught it.
// =========================================================================

#[test]
fn gangnam_style_i32_overflow() {
    let views = Mi::new(2_147_483_648); // one past i32::MAX

    // Can't fit in i32
    assert!(views.narrow_i32().is_err());

    // But the exact strand is fine
    assert_eq!(views.collapse(), BigInt::from(2_147_483_648i64));

    // i32::MAX itself is fine
    let max_views = Mi::new(2_147_483_647);
    assert_eq!(max_views.narrow_i32().unwrap(), 2_147_483_647);
}

// =========================================================================
// Boeing 787 — April 2015
//
// A 32-bit counter in the power control unit counted centiseconds.
// After 248 days of continuous operation (2^31 centiseconds),
// the counter overflowed. All AC power could be lost.
// FAA issued an Airworthiness Directive requiring periodic reboots.
//
// With MöbiusInteger, the corruption would be detectable.
// =========================================================================

#[test]
fn boeing_787_timer_overflow() {
    // 2^31 centiseconds = 2,147,483,648 cs
    // = 248.55 days
    let centiseconds_per_day: i64 = 100 * 60 * 60 * 24; // 8,640,000

    // Simulate 249 days of uptime in one go
    let timer = Mi::new(centiseconds_per_day * 249);

    // This exceeds i32 range
    assert!(timer.narrow_i32().is_err());

    // But still fits in i64, so machine strand is fine
    assert!(!timer.is_corrupted());

    // The exact value is knowable
    assert_eq!(timer.collapse(), BigInt::from(centiseconds_per_day * 249));
}

// =========================================================================
// Construction
// =========================================================================

#[test]
fn new_strands_agree() {
    let n = Mi::new(42);
    assert_eq!(n.machine(), 42);
    assert_eq!(n.exact(), &BigInt::from(42));
    assert!(!n.is_corrupted());
}

#[test]
fn from_big_fits() {
    let n = Mi::from_big(BigInt::from(1000));
    assert_eq!(n.machine(), 1000);
    assert!(!n.is_corrupted());
}

#[test]
fn from_big_saturates() {
    let huge = BigInt::from(i64::MAX) + BigInt::from(100);
    let n = Mi::from_big(huge.clone());
    assert_eq!(n.machine(), i64::MAX); // saturated
    assert_eq!(n.exact(), &huge);
    assert!(n.is_corrupted());
}

#[test]
fn from_conversions() {
    let a: Mi = 42i64.into();
    let b: Mi = 42i32.into();
    let c: Mi = 42i16.into();
    let d: Mi = 42u32.into();
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(c, d);
}

// =========================================================================
// Arithmetic
// =========================================================================

#[test]
fn add_normal() {
    let result = Mi::new(100) + Mi::new(200);
    assert_eq!(result.collapse(), BigInt::from(300));
    assert!(!result.is_corrupted());
}

#[test]
fn sub_normal() {
    let result = Mi::new(500) - Mi::new(200);
    assert_eq!(result.collapse(), BigInt::from(300));
}

#[test]
fn mul_normal() {
    let result = Mi::new(7) * Mi::new(6);
    assert_eq!(result.collapse(), BigInt::from(42));
}

#[test]
fn div_normal() {
    let result = Mi::new(42) / Mi::new(6);
    assert_eq!(result.collapse(), BigInt::from(7));
}

#[test]
fn rem_normal() {
    let result = Mi::new(10) % Mi::new(3);
    assert_eq!(result.collapse(), BigInt::from(1));
}

#[test]
fn neg_normal() {
    let result = -Mi::new(42);
    assert_eq!(result.collapse(), BigInt::from(-42));
}

#[test]
fn add_wraps_machine_but_exact_survives() {
    let a = Mi::new(i64::MAX);
    let b = Mi::new(i64::MAX);
    let result = a + b;

    // Machine wrapped
    assert_eq!(result.machine(), -2); // wrapping: MAX + MAX = -2 in i64
    assert!(result.is_corrupted());

    // Exact is correct
    let expected = BigInt::from(i64::MAX) + BigInt::from(i64::MAX);
    assert_eq!(result.collapse(), expected);
}

#[test]
fn mul_overflow_detected() {
    let a = Mi::new(i64::MAX);
    let b = Mi::new(2);
    let result = a * b;

    assert!(result.is_corrupted());
    assert_eq!(result.collapse(), BigInt::from(i64::MAX) * BigInt::from(2));
}

#[test]
fn neg_min_overflow() {
    // Negating i64::MIN overflows because |i64::MIN| > i64::MAX
    let result = -Mi::new(i64::MIN);
    assert!(result.is_corrupted());
    assert_eq!(result.machine(), i64::MIN); // wrapping_neg of MIN is MIN
    assert_eq!(result.collapse(), -BigInt::from(i64::MIN));
}

// =========================================================================
// Reference arithmetic
// =========================================================================

#[test]
fn add_by_ref() {
    let a = Mi::new(100);
    let b = Mi::new(200);
    let result = &a + &b;
    assert_eq!(result.collapse(), BigInt::from(300));
    // a and b are still usable
    assert_eq!(a.machine(), 100);
    assert_eq!(b.machine(), 200);
}

// =========================================================================
// Comparison — always exact
// =========================================================================

#[test]
fn eq_uses_exact() {
    let a = Mi::new(42);
    let b = Mi::new(42);
    assert_eq!(a, b);
}

#[test]
fn ordering_uses_exact() {
    let a = Mi::new(1);
    let b = Mi::new(2);
    assert!(a < b);
    assert!(b > a);
}

#[test]
fn corrupted_values_compare_correctly() {
    // Two values where machine strands are equal (both wrapped to same value)
    // but exact strands differ
    let a = Mi::new(i64::MAX) + Mi::new(1); // machine: MIN, exact: MAX+1
    let b = Mi::new(i64::MIN); // machine: MIN, exact: MIN

    // Machine strands are equal
    assert_eq!(a.machine(), b.machine());

    // But MöbiusInteger knows a > b
    assert!(a > b);
    assert_ne!(a, b);
}

// =========================================================================
// Display
// =========================================================================

#[test]
fn display_normal() {
    let n = Mi::new(42);
    assert_eq!(format!("{}", n), "42");
}

#[test]
fn display_corrupted() {
    let result = Mi::new(i64::MAX) + Mi::new(1);
    let s = format!("{}", result);
    assert!(s.contains("CORRUPTED"));
    assert!(s.contains("9223372036854775808"));
}

// =========================================================================
// Accumulation — death by a thousand additions
// =========================================================================

#[test]
fn accumulate_past_max() {
    let mut total = Mi::new(i64::MAX - 10);
    for _ in 0..20 {
        total = total + Mi::new(1);
    }

    // Machine strand wrapped
    assert!(total.is_corrupted());

    // Exact strand is correct
    assert_eq!(
        total.collapse(),
        BigInt::from(i64::MAX) + BigInt::from(10)
    );
}

#[test]
fn accumulate_no_overflow() {
    let mut total = Mi::new(0);
    for i in 0..1000 {
        total = total + Mi::new(i);
    }
    // Sum of 0..999 = 999 * 1000 / 2 = 499500
    assert_eq!(total.collapse(), BigInt::from(499_500));
    assert!(!total.is_corrupted());
}
