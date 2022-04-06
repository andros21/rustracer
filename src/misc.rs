pub const EPSILON: f32 = 1e-4;

pub trait IsClose<Rhs = Self> {
    fn is_close(&self, other: Self) -> bool;
}

impl IsClose for f32 {
    fn is_close(&self, other: f32) -> bool {
        (self - other).abs() < EPSILON
    }
}

pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_close_float() {
        assert!((EPSILON * 1e-1 + 1.0).is_close(1.0));
        assert!(!(EPSILON + 1.0).is_close(1.0))
    }
}
