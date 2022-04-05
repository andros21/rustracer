pub const EPSILON: f32 = 1e-5;

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
