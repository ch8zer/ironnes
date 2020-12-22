#[derive(Default)]
pub struct BitSet(u8);

fn bit_toggle(v: u8, state: u8, posn: u8) -> u8 {
    let x = state & 1;
    let clr = v & !(1 << posn);
    clr | (x << posn)
}

fn bit_get(v: u8, posn: u8) -> bool {
    v & (1 << posn) != 0
}

impl BitSet {
    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn set(&mut self, idx: u8, val: u8) {
        self.0 = bit_toggle(self.0, val, idx);
    }

    pub fn get(&self, idx: u8) -> bool {
        bit_get(self.0, idx)
    }

    pub fn cast(&self) -> u8 {
        self.0
    }
}

/**
 * A biased bitset is one where not every bit can be toggled,
 * some bits are tied to 0/1 at all times.
 * This can be used to similate common hardware registers
 * that have certain bits tied to ground/power.
 */
pub struct BiasedBitSet {
    v: u8,
    set0: u8,
    set1: u8,
}

impl BiasedBitSet {
    pub fn store(&mut self, v: u8) {
        self.v = self.sanitize(v);
    }

    pub fn bias(&mut self, bit: u8, v: u8) {
        let v = v & 1;
        self.set0 = self.set0 & !((v ^ 1) << bit);
        self.set1 = self.set1 | (v << bit);
        self.v = self.sanitize(self.v);
    }

    pub fn set(&mut self, idx: u8, val: u8) {
        self.v = self.sanitize(bit_toggle(self.v, val, idx));
    }

    pub fn get(&self, idx: u8) -> bool {
        bit_get(self.v, idx)
    }

    pub fn cast(&self) -> u8 {
        self.v
    }

    /**
     * Sanitizes the input, tying it to the "broken" bits.
     * All bits are stored broken
     */
    fn sanitize(&self, v: u8) -> u8 {
        (v & self.set0) | self.set1
    }
}

impl Default for BiasedBitSet {
    fn default() -> Self {
        Self {
            v: 0,
            set0: 0xff,
            set1: 0,
        }
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
        bs.set(3, 0);
        bs.set(5, 1);
        assert_eq!(0b11110001, bs.cast());

        [1, 1, 1, 1, 0, 0, 0, 1]
            .iter()
            .rev()
            .enumerate()
            .for_each(|(i, v)| assert_eq!((*v != 0), bs.get(i as u8)));
    }

    #[test]
    fn test_biased_bitset_getset() {
        let mut bs = BiasedBitSet::default();
        bs.bias(3, 0);
        bs.bias(5, 1);
        bs.store(0b11011001);
        assert_eq!(0b11110001, bs.cast());
        bs.set(3, 0);
        bs.set(5, 1);
        assert_eq!(0b11110001, bs.cast());
    }
}
