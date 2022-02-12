use crate::frontend::tokens::{Token, TokenData};

pub struct ScopeIter<I> {
    iter: I,
    level: usize,
}

impl<I> ScopeIter<I> {
    pub fn new(iter: I) -> Self {
        Self { iter, level: 1 }
    }
}

impl<I> Iterator for ScopeIter<I>
where
    I: Iterator<Item = Token>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.level == 0 {
            return None;
        }

        let next = self.iter.next()?;

        match &next.0 {
            TokenData::OpenCurly => {
                self.level += 1;
            }
            TokenData::CloseCurly => {
                self.level -= 1;

                if self.level == 0 {
                    return None;
                }
            }
            _ => {}
        };

        Some(next)
    }
}
