pub struct BitSet(u8);

impl BitSet {
    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn set(&mut self, idx: u8) {
        self.0 = self.0 | (1 << idx)
    }

    pub fn clear(&mut self, idx: u8) {
        self.0 = self.0 & !(1 << idx)
    }

    pub fn get(&self, idx: u8) -> bool {
        ((1 << idx) & self.0) != 0
    }

    pub fn cast(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitset_sanity() {
        let bs = BitSet::new(0b11011001);
        assert_eq!(0b11011001, bs.cast());
    }

    #[test]
    fn test_bitset_getset() {
        let mut bs = BitSet::new(0b11011001);
        bs.clear(3);
        bs.set(5);
        assert_eq!(0b11110001, bs.cast());

        [1, 1, 1, 1, 0, 0, 0, 1]
            .iter()
            .rev()
            .enumerate()
            .for_each(|(i, v)| assert_eq!((*v != 0), bs.get(i as u8)));
    }
}
