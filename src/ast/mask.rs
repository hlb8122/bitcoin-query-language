pub struct Mask(Vec<bool>);

impl Mask {
    pub fn invert(&mut self) -> Self {
        Mask(self.0.iter().map(|b| !b).collect())
    }

    pub fn apply(&self, other: Vec<u8>) -> Vec<u8> {
        other
            .iter()
            .enumerate()
            .filter_map(|(i, byte)| match self.0.get(i) {
                Some(true) => Some(*byte),
                _ => None,
            })
            .collect()
    }
}
