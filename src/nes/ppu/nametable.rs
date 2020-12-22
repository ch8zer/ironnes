#[derive(Default)]
pub struct NameTable {
    data: [u8; 1024]
}

impl NameTable {
    const NUM_COLS: usize = 32;
    const NUM_ROWS: usize = 30;

    // Walk the nametable entries
    pub fn iter(&self) -> impl Iterator<(&u8, (usize, usize), &u8)> {

    }
    

}
