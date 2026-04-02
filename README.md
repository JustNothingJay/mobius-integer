# MöbiusInteger

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19154835.svg)](https://doi.org/10.5281/zenodo.19154835)

**Every systems language gets this wrong:**

```
i64::MAX + 1 = i64::MIN    // Rust (wrapping)
2147483647 + 1 = -2147483648  // C, Java, Go
```

The Ariane 5 rocket exploded because of it. YouTube broke because of it. Boeing issued emergency reboots because of it.

**MöbiusInteger fixes it:**

```rust
use mobius_integer::Mi;

let max = Mi::new(i64::MAX);
let result = max + Mi::new(1);

assert_eq!(result.machine(), i64::MIN);  // machine wrapped — this is what hardware does
assert!(result.is_corrupted());           // but now you KNOW it wrapped
assert_eq!(result.collapse().to_string(), "9223372036854775808"); // truth
```

## How it works

Every MöbiusInteger stores two strands:

- **Machine strand** (`i64`): hardware-fast, uses wrapping arithmetic — behaves exactly like C/Java/Go
- **Exact strand** (`BigInt`): arbitrary precision, never overflows — carries the truth

Arithmetic propagates both strands simultaneously. Comparison and collapse always use the exact strand. The overflow still *happens* on the machine strand, but it's never consulted for truth.

The name comes from the Möbius strip — one surface, no front or back. The number and its correction are the same object.

## The disasters, all caught

### Ariane 5 — June 4, 1996

A 64-bit float was narrowed to a 16-bit signed integer. The value exceeded 32,767. The conversion overflowed silently. The rocket self-destructed 37 seconds after launch. $370 million payload lost.

```rust
let velocity = Mi::new(32_768);

// Old way: silent truncation, rocket explodes
// MöbiusInteger way:
assert!(velocity.narrow_i16().is_err());  // caught. rocket lives.
```

### YouTube / Gangnam Style — December 2014

PSY's video exceeded 2,147,483,647 views. YouTube's 32-bit counter couldn't hold it.

```rust
let views = Mi::new(2_147_483_648);
assert!(views.narrow_i32().is_err());  // caught before the counter breaks
```

### Boeing 787 — April 2015

A 32-bit centisecond counter overflowed after 248 days of continuous operation. The FAA ordered periodic reboots.

```rust
let timer = Mi::new(100 * 60 * 60 * 24 * 249);  // 249 days in centiseconds
assert!(timer.narrow_i32().is_err());  // caught. no reboot needed.
```

## The corruption detector

```rust
let a = Mi::new(i64::MAX);
let b = Mi::new(i64::MAX);
let result = a + b;

println!("{}", result);
// Output: 18446744073709551614 (machine: -2, CORRUPTED)

result.is_corrupted()  // true — the machine strand is lying
result.collapse()      // BigInt: 18446744073709551614 — the truth
result.machine()       // -2 — what C would have given you
```

## API

| Method | Returns | Description |
|--------|---------|-------------|
| `Mi::new(i64)` | `MobiusInteger` | Create from i64, both strands agree |
| `Mi::from_big(BigInt)` | `MobiusInteger` | Create from BigInt, machine saturates |
| `.machine()` | `i64` | The fast strand (may have wrapped) |
| `.exact()` | `&BigInt` | The truth strand (never overflows) |
| `.collapse()` | `BigInt` | Möbius traversal — returns exact |
| `.is_corrupted()` | `bool` | True if machine ≠ exact |
| `.narrow_i16()` | `Result<i16, OverflowError>` | Safe narrowing (Ariane 5 fix) |
| `.narrow_i32()` | `Result<i32, OverflowError>` | Safe narrowing (Gangnam Style fix) |
| `.narrow_i64()` | `Result<i64, OverflowError>` | Safe narrowing |
| `+ - * / % -` | `MobiusInteger` | All propagate both strands |
| `== != < > <= >=` | `bool` | All use exact strand |

## Install

```toml
[dependencies]
mobius-integer = "0.1"
```

## Part of the Möbius family

- **[mobius-number](https://github.com/JustNothingJay/mobius-number)** — fixes IEEE 754 floating point (`0.1 + 0.2 = 0.3`)
- **[mobius-constant](https://github.com/JustNothingJay/mobius-constant)** — fixes irrational identity (`sqrt(2)**2 = 2`)
- **[mobius-units](https://github.com/JustNothingJay/mobius-units)** — derives fundamental constants from the eigenvalue tower
- **mobius-integer** — fixes integer overflow (this crate)

Same pattern. Same anatomy. Same fix. Different domain.

## License

MIT
