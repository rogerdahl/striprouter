use crate::via::Via;

// #[derive(Eq, PartialEq, PartialOrd, Clone, Copy)]
#[derive(Clone, Copy)]
pub(crate) struct Board {
    pub w: usize,
    pub h: usize,
}

impl Board {
    pub(crate) fn new(w: usize, h: usize) -> Self {
        Self { w, h }
    }

    pub fn size(&self) -> usize {
        self.w * self.h
    }

    // fn idx(&self, x: usize, y: usize) -> usize {
    //     y * self.w + x
    // }

    pub(crate) fn idx(&self, via: Via) -> usize {
        via.y * self.w + via.x
    }
}
