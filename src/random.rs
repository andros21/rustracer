//! [PCG](https://www.pcg-random.org/index.html) random numbers generator module.
//!
//! Provides [`Pcg`](struct@Pcg) struct.

/// [PCG](https://www.pcg-random.org/index.html) random numbers generator.
#[derive(Copy, Clone, Debug)]
pub struct Pcg {
    state: u64,
    inc: u64,
}

impl Default for Pcg {
    /// Provides a default constructor for [`Pcg`](struct@Pcg),
    /// the default values for the seed are:
    /// - `init_state=42`;
    /// - `init_seq=54`.
    fn default() -> Self {
        Self::new(42, 54)
    }
}

impl Pcg {
    /// Provides a constructor for [`Pcg`](struct@Pcg).
    ///
    /// `init_state` and `init_seq` are the seed of the generator.
    pub fn new(init_state: u64, init_seq: u64) -> Self {
        let state = 0;
        let inc = (init_seq << 1) | 1;
        let mut pcg = Pcg { state, inc };
        pcg.random();
        pcg.state += init_state;
        pcg.random();
        pcg
    }

    /// Generates an integer random number.
    pub fn random(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state.wrapping_mul(6364136223846793005) + self.inc;
        let xor_shifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = old_state >> 59;
        (xor_shifted >> rot) as u32 | (xor_shifted << ((-(rot as i64)) & 31))
    }

    /// Generates a float random number.
    pub fn random_float(&mut self) -> f32 {
        self.random() as f32 / u32::MAX as f32
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_random() {
        let mut pcg = Pcg::default();
        assert_eq!(pcg.state, 1753877967969059832);
        assert_eq!(pcg.inc, 109);
        for expected in [
            2707161783, 2068313097, 3122475824, 2211639955, 3215226955, 3421331566,
        ] {
            assert_eq!(pcg.random(), expected);
        }
    }

    #[test]
    fn test_random_float() {
        let mut pcg = Pcg::new(38, 62);

        for expected in [
            0.09002101213904587,
            0.3903793735407245,
            0.664116223730174,
            0.42459877776554755,
            0.30006475823467244,
            0.15857429922525174,
        ] {
            assert_eq!(pcg.random_float(), expected);
        }
    }
}
