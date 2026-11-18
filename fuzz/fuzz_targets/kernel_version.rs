#![no_main]

use libfuzzer_sys::fuzz_target;

// Compile the production parser directly without introducing a library API.
#[allow(dead_code)]
#[path = "../../src/kernel.rs"]
mod kernel;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let actual = kernel::KernelVersion::parse(&input);

    // Independent byte-wise oracle for the documented first-three-token rule.
    // Missing or overflowing u32 components become zero, and suffixes are ignored.
    let mut expected = [0_u32; 3];
    let mut component = 0;
    let mut digits = false;
    let mut value = Some(0_u32);
    for byte in input.bytes().chain(std::iter::once(b'.')) {
        if byte.is_ascii_digit() {
            digits = true;
            value = value.and_then(|v| v.checked_mul(10)?.checked_add(u32::from(byte - b'0')));
        } else if digits {
            expected[component] = value.unwrap_or(0);
            component += 1;
            if component == expected.len() {
                break;
            }
            digits = false;
            value = Some(0);
        }
    }
    assert_eq!([actual.major, actual.minor, actual.patch], expected);

    // Display output must round-trip and the actual gate must agree with ordering.
    assert_eq!(kernel::KernelVersion::parse(&actual.to_string()), actual);
    assert!(actual.at_least(actual.major, actual.minor));
    assert_eq!(
        actual.at_least(3, 17),
        actual
            >= kernel::KernelVersion {
                major: 3,
                minor: 17,
                patch: 0
            }
    );
});
