//! Helper functions.

/// Read a buffer smaller than 8 bytes into an integer in little-endian.
///
/// This assumes that `buf.len() < 8`. If this is not satisfied, the behavior is unspecified.
#[inline(always)]
pub fn read_int(buf: &[u8]) -> u64 {
    // Because we want to make sure that it is register allocated, we fetch this into a variable.
    // It will likely make no difference anyway, though.
    let ptr = buf.as_ptr();

    unsafe {
        // Break it down to reads of integers with widths in total spanning the buffer. This minimizes
        // the number of reads
        // The right-most match will also be 0..4 due to the size of a u32
        match (buf.len(), ptr as usize & core::mem::align_of::<u32>()) {
            // u8.
            // We don't case about our alignment because we just read the pointer
            (1, _) => *ptr as u64,
            // u16.
            // For a u16, even if its unaligned we can't do any better
            (2, _) => (ptr as *const u16).read_unaligned().to_le() as u64,
            // If our alignment is even, then read the u16 first
            (3, x) if x & 1 == 0 => {
                // u16 + u8.
                let a = (ptr as *const u16).read_unaligned().to_le() as u64;
                let b = *ptr.offset(2) as u64;

                a | (b << 16)
            }
            // Otherwise read the u8 first to align the u16 read
            (3, _) => {
                // u8 + u16
                let a = *ptr as u64;
                let b = (ptr.offset(1) as *const u16).read_unaligned().to_le() as u64;

                a | (b << 8)
            }
            // u32.
            (4, 0) => (ptr as *const u32).read_unaligned().to_le() as u64,
            (4, 2) => {
                let a = (ptr as *const u16).read_unaligned().to_le() as u64;
                let b = (ptr.offset(2) as *const u16).read_unaligned().to_le() as u64;

                a | (b << 16)
            }
            (4, _) => (ptr as *const u32).read_unaligned().to_le() as u64,
            // u32 + u8.
            (5, 0) => {
                let a = (ptr as *const u32).read_unaligned().to_le() as u64;
                let b = *ptr.offset(4) as u64;

                a | (b << 32)
            }
            (5, 1) => {
                // u8 + u16 + u16
                let a = *ptr as u64;
                let b = (ptr.offset(1) as *const u16).read_unaligned().to_le() as u64;
                let c = (ptr.offset(3) as *const u16).read_unaligned().to_le() as u64;

                a | (b << 8) | (c << 24)
            }
            (5, 2) => {
                // u16 + u16 + u8
                let a = (ptr as *const u16).read_unaligned().to_le() as u64;
                let b = (ptr.offset(2) as *const u16).read_unaligned().to_le() as u64;
                let c = *ptr.offset(4) as u64;

                a | (b << 16) | (c << 32)
            }
            (5, _) => {
                // u8 + u32
                let a = *ptr as u64;
                let b = (ptr.offset(1) as *const u32).read_unaligned().to_le() as u64;

                a | (b << 8)
            }
            (6, 0) => {
                // u32 + u16
                let a = (ptr as *const u32).read_unaligned().to_le() as u64;
                let b = (ptr.offset(4) as *const u16).read_unaligned().to_le() as u64;

                a | (b << 32)
            }
            (6, 1) => {
                // u8 + u16 + u16 + u8
                let a = *ptr as u64;
                let b = (ptr.offset(1) as *const u16).read_unaligned().to_le() as u64;
                let c = (ptr.offset(3) as *const u16).read_unaligned().to_le() as u64;
                let d = *ptr.offset(5) as u64;

                a | (b << 8) | (c << 24) | (d << 40)
            }
            (6, 2) => {
                // u16 + u32
                let a = (ptr as *const u16).read_unaligned().to_le() as u64;
                let b = (ptr.offset(4) as *const u32).read_unaligned().to_le() as u64;

                a | (b << 16)
            }
            (6, _) => {
                // u8 + u32 + u8
                let a = *ptr as u64;
                let b = (ptr.offset(1) as *const u32).read_unaligned().to_le() as u64;
                let c = *ptr.offset(5) as u64;

                a | (b << 8) | (c << 40)
            }
            (7, 0) => {
                // u32 + u16 + u8.
                let a = (ptr as *const u32).read_unaligned().to_le() as u64;
                let b = (ptr.offset(4) as *const u16).read_unaligned().to_le() as u64;
                let c = *ptr.offset(6) as u64;

                a | (b << 32) | (c << 48)
            }
            (7, 1) => {
                // u8 + u16 + u32.
                let a = *ptr as u64;
                let b = (ptr.offset(1) as *const u16).read_unaligned().to_le() as u64;
                let c = (ptr.offset(3) as *const u32).read_unaligned().to_le() as u64;

                a | (b << 8) | (c << 24)
            }
            (7, 2) => {
                // u16 + u32 + u8.
                let a = (ptr as *const u16).read_unaligned().to_le() as u64;
                let b = (ptr.offset(2) as *const u32).read_unaligned().to_le() as u64;
                let c = *ptr.offset(6) as u64;

                a | (b << 16) | (c << 48)
            }
            (7, _) => {
                // u8 + u32 + u16.
                let a = *ptr as u64;
                let b = (ptr.offset(1) as *const u32).read_unaligned().to_le() as u64;
                let c = (ptr.offset(5) as *const u16).read_unaligned().to_le() as u64;

                a | (b << 8) | (c << 40)
            }
            _ => 0,
        }
    }
}

/// Read a little-endian 64-bit integer from some buffer.
#[inline(always)]
pub unsafe fn read_u64(ptr: *const u8) -> u64 {
    #[cfg(target_pointer_width = "32")]
    {
        // We cannot be sure about the memory layout of a potentially emulated 64-bit integer, so
        // we read it manually. If possible, the compiler should emit proper instructions.
        let a = (ptr as *const u32).read_unaligned().to_le();
        let b = (ptr.offset(4) as *const u32).read_unaligned().to_le();

        a as u64 | ((b as u64) << 32)
    }

    #[cfg(target_pointer_width = "64")]
    {
        (ptr as *const u64).read_unaligned().to_le()
    }
}

/// The diffusion function.
///
/// This is a bijective function emitting chaotic behavior. Such functions are used as building
/// blocks for hash functions.
pub const fn diffuse(mut x: u64) -> u64 {
    // These are derived from the PCG RNG's round. Thanks to @Veedrac for proposing this. The basic
    // idea is that we use dynamic shifts, which are determined by the input itself. The shift is
    // chosen by the higher bits, which means that changing those flips the lower bits, which
    // scatters upwards because of the multiplication.

    x = x.wrapping_mul(0x6eed0e9da4d94a4f);
    let a = x >> 32;
    let b = x >> 60;
    x ^= a >> b;
    x = x.wrapping_mul(0x6eed0e9da4d94a4f);

    x
}

/// Reverse the `diffuse` function.
pub const fn undiffuse(mut x: u64) -> u64 {
    // 0x2f72b4215a3d8caf is the modular multiplicative inverse of the constant used in `diffuse`.

    x = x.wrapping_mul(0x2f72b4215a3d8caf);
    let a = x >> 32;
    let b = x >> 60;
    x ^= a >> b;
    x = x.wrapping_mul(0x2f72b4215a3d8caf);

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diffuse_test(x: u64, y: u64) {
        assert_eq!(diffuse(x), y);
        assert_eq!(x, undiffuse(y));
        assert_eq!(undiffuse(diffuse(x)), x);
    }

    #[test]
    fn read_int_() {
        assert_eq!(read_int(&[2, 3]), 770);
        assert_eq!(read_int(&[3, 2]), 515);
        assert_eq!(read_int(&[3, 2, 5]), 328195);
    }

    #[test]
    fn read_u64_() {
        unsafe {
            assert_eq!(read_u64([1, 0, 0, 0, 0, 0, 0, 0].as_ptr()), 1);
            assert_eq!(read_u64([2, 1, 0, 0, 0, 0, 0, 0].as_ptr()), 258);
        }
    }

    #[test]
    fn diffuse_test_vectors() {
        diffuse_test(94203824938, 17289265692384716055);
        diffuse_test(0xDEADBEEF, 12110756357096144265);
        diffuse_test(0, 0);
        diffuse_test(1, 15197155197312260123);
        diffuse_test(2, 1571904453004118546);
        diffuse_test(3, 16467633989910088880);
    }
}
