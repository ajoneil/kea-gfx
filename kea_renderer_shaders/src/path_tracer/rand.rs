// Xoroshiro128+
#[derive(Clone, Debug)]
pub struct Random {
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
}

impl Random {
    pub fn new(s0: u32, s1: u32, s2: u32, s3: u32) -> Self {
        let mut rng = Self { s0, s1, s2, s3 };
        rng.next_u32();
        rng
    }

    pub fn next_float(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn next_u32(&mut self) -> u32 {
        let result = self.s0 + self.s3;
        let t = self.s1 << 9;

        self.s2 ^= self.s0;
        self.s3 ^= self.s1;
        self.s1 ^= self.s2;
        self.s0 ^= self.s3;

        self.s2 ^= t;

        self.s3 = (self.s3 << 11) | (self.s3 >> 21);

        result
    }
}
