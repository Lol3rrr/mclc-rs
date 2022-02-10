#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    // The Cell
    x: usize,
    // The Row
    y: usize,
    // The Layer
    z: usize,
}

impl Position {
    pub fn left(&self) -> Option<Self> {
        todo!("")
    }
    pub fn right(&self) -> Option<Self> {
        todo!("")
    }
    pub fn up(&self) -> Option<Self> {
        todo!("")
    }
    pub fn down(&self) -> Option<Self> {
        todo!("")
    }
    pub fn above(&self) -> Option<Self> {
        todo!("")
    }
    pub fn below(&self) -> Option<Self> {
        todo!("")
    }
}
