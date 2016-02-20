#![feature(plugin)]
#![plugin(clippy)]

#![feature(test)]
extern crate test;

#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate log;

extern crate num;
extern crate roaring;
extern crate rayon;
extern crate rand;

pub use grid::{Cell, CellType};
use grid::Grid;

 mod grid;

pub const PAGE_SIZE: u32 = 65536;
pub const PAGE_WIDTH: u32 = 256;

trait ReportMemory {
    fn memory(&self) -> u32;
}


pub struct Cajal {
    grid: Grid,
}

impl Default for Cajal {
    fn default() -> Cajal {
        Cajal { grid: Grid::default() }
    }
}

impl ReportMemory for Cajal {
    fn memory(&self) -> u32 {
        self.grid.memory()
    }
}

impl Cajal {
    pub fn new(size: u32, density: f32, seed: &[usize]) -> Cajal {
        Cajal { grid: Grid::new(size, density, seed) }
    }

    pub fn grow(&mut self) {
        self.grid.grow();
    }

    pub fn grow_step(&mut self) -> u32 {
        self.grid.grow_step()
    }

    pub fn get_cell(&self, x: u32, y: u32) -> &Cell {
        self.grid.get_cell(x, y)
    }

    pub fn signal(&mut self) {
        self.grid.signal();
    }

    pub fn signal_step(&mut self) -> u32 {
        self.grid.signal_step()
    }

    pub fn set_input(&mut self, x: u32, y: u32, sig: u8) {
        self.grid.set_input(x, y, sig);
    }
}





#[cfg(test)]
mod tests {
    use super::Cajal;
    use test::Bencher;

    #[test]
    fn default_params() {
        let _ = Cajal::default();
    }

    #[bench]
    fn bench_new_5x5(b: &mut Bencher) {
        b.iter(|| {
            Cajal::new(5, 0.05, &[1, 2, 3, 4]);
        });
    }

    #[bench]
    fn bench_new_2x2(b: &mut Bencher) {
        b.iter(|| {
            Cajal::new(2, 0.05, &[1, 2, 3, 4]);
        });
    }
}
