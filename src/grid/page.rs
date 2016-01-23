
use std::slice::IterMut;
use roaring::RoaringBitmap;
use std::collections::HashMap;
use rayon::par_iter::*;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;


pub use super::cell::{Cell, Chromosome, CellType, Gate};
use super::zorder;
use super::super::ReportMemory;

pub enum GrowthPhase {
    Axon,
    Dendrite
}

pub struct Page {
    cells: Vec<Cell>,
    active: RoaringBitmap<u32>,
    changes: HashMap<u32, Cell>
}

impl ReportMemory for Page {
    fn memory(&self) -> u32 {
        println!(".");
        (self.cells.len() as u32 * 16) +
        (self.active.len() as u32 * 8) +  // <-- This is not true!
        ((self.changes.len() as u32 + self.changes.capacity() as u32) * 8) // Rough approximation
    }
}

impl Page {
    pub fn new(density: f32) -> Page {
        let mut rng = thread_rng();

        let num_cells = 4096;
        let mut cells = Vec::with_capacity(num_cells);

        for _ in 0..num_cells {
            let mut cell = Cell::new();
            cell.set_chromosome(rng.gen());
            cells.push(cell);
        }

        let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();

        // TODO roll this into the initialization loop
        let active_cells: u32 = (4096f32 * density).round() as u32;

        let range_cells = Range::new(0, num_cells - 1);
        let range_gate = Range::new(0, 4);

        for _ in 0..active_cells {
            let index = range_cells.ind_sample(&mut rng);
            cells[index].set_cell_type(CellType::Body);
            cells[index].set_gate(rng.gen());
            bitmap.insert(index as u32);
        }

        Page {
            cells: cells,
            active: bitmap,
            changes: HashMap::new()
        }
    }

    pub fn grow(&mut self, phase: GrowthPhase) -> u32 {
        if self.active.is_empty() == true {
            return 0;
        }

        let cell_type = match phase {
            GrowthPhase::Axon => CellType::Axon,
            GrowthPhase::Dendrite => CellType::Dendrite
        };

        for index in self.active.iter() {
            let mut cells = &mut self.cells;
            let (mut x, mut y) = zorder::z_to_xy(index);

            // Explicitly going to clobber existing changes for simplicity
            if cells[index as usize].chromosome_contains(Chromosome::North) {
                if y < 63 {
                    if let Some(change) = Page::grow_local(cells, x, y, cell_type, Chromosome::North) {
                        self.changes.insert(index, change);
                    }
                }
            }

            if cells[index as usize].chromosome_contains(Chromosome::South) {
                if y > 0 {
                    if let Some(change) = Page::grow_local(cells, x, y, cell_type, Chromosome::South) {
                        self.changes.insert(index, change);
                    }
                }
            }

            if cells[index as usize].chromosome_contains(Chromosome::East) {
                if x < 63 {
                    if let Some(change) = Page::grow_local(cells, x, y, cell_type, Chromosome::East) {
                        self.changes.insert(index, change);
                    }
                }
            }

            if cells[index as usize].chromosome_contains(Chromosome::West) {
                if x > 0 {
                    if let Some(change) = Page::grow_local(cells, x, y, cell_type, Chromosome::West) {
                        self.changes.insert(index, change);
                    }
                }
            }
        }

        // Clear out the active cell bitmap, and add the cells we just grew
        self.active.clear();
        for (k, v) in &self.changes {
            self.active.insert(*k);
        }

        // Return the number of newly activated cells
        self.changes.len() as u32
    }

    pub fn update(&self) {
        if self.changes.len() == 0 {
            return;
        }

    }

    // TODO use i64 instead, so we can check for accidental negatives?
    fn grow_local(cells: &mut Vec<Cell>, x: u32, y: u32, cell_type: CellType, direction: Chromosome) -> Option<Cell> {
        println!("({},{})", x, y);
        assert!((x > 63 && direction == Chromosome::East) != true);
        assert!((y > 63 && direction == Chromosome::North) != true);

        let (target, gate) = match direction {
            Chromosome::North => (zorder::xy_to_z(x, y + 1), Gate::South),
            Chromosome::South => (zorder::xy_to_z(x, y - 1), Gate::North),
            Chromosome::East => (zorder::xy_to_z(x + 1, y), Gate::West),
            Chromosome::West => (zorder::xy_to_z(x - 1, y), Gate::East),
            _ => unreachable!()
        };

        Page::create_change(&mut cells[target as usize], cell_type, gate)
    }

    fn create_change(cell: &mut Cell, cell_type: CellType, gate: Gate) -> Option<Cell>{
        if cell.get_cell_type() == CellType::Empty {
            // TODO reuse from a pool of allocated cells?
            let mut change = Cell::new();
            change.set_cell_type(cell_type);
            change.set_gate(gate);
            return Some(change);
        }
        None
    }
}


#[cfg(test)]
mod test {
    use super::{Page, GrowthPhase, Cell, CellType, Gate, Chromosome};

    #[test]
    fn page_new() {
        let p = Page::new(0.05);
    }

    #[test]
    fn grow() {
        let mut p = Page::new(0.05);
        p.grow(GrowthPhase::Axon);
    }

    #[test]
    fn create_change_empty() {
        let mut cell = Cell::new();
        assert!(cell.get_cell_type() == CellType::Empty);
        assert!(cell.get_gate() == Gate::North);

        let change = Page::create_change(&mut cell, CellType::Axon, Gate::North);
        assert!(change.is_some() == true);

        let change = change.unwrap();
        assert!(change.get_cell_type() == CellType::Axon);
        assert!(change.get_gate() == Gate::North);
    }

    #[test]
    fn create_change_non_empty() {
        let mut cell = Cell::new();
        assert!(cell.get_cell_type() == CellType::Empty);
        assert!(cell.get_gate() == Gate::North);

        cell.set_cell_type(CellType::Dendrite);
        cell.set_gate(Gate::South);
        assert!(cell.get_cell_type() == CellType::Dendrite);
        assert!(cell.get_gate() == Gate::South);

        let change = Page::create_change(&mut cell, CellType::Axon, Gate::North);
        assert!(change.is_none() == true);

        assert!(cell.get_cell_type() == CellType::Dendrite);
        assert!(cell.get_gate() == Gate::South);
    }

    #[test]
    fn grow_local() {
        let mut data = vec![Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        let change = Page::grow_local(&mut data, 0, 0, CellType::Axon, Chromosome::North);
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        assert!(change.is_some() == true);
        let change = change.unwrap();
        assert!(change.get_cell_type() == CellType::Axon);
        assert!(change.get_gate() == Gate::South); // Gate is opposite of the growth direction

        let change = Page::grow_local(&mut data, 1, 0, CellType::Dendrite, Chromosome::West);
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        assert!(change.is_some() == true);
        let change = change.unwrap();
        assert!(change.get_cell_type() == CellType::Dendrite);
        assert!(change.get_gate() == Gate::East);   // Gate is opposite of the growth direction
    }


    #[test]
    #[should_panic]
    fn grow_local_bad_north() {
        let mut data = vec![Cell::new(), Cell::new()];
        let change = Page::grow_local(&mut data, 0, 63, CellType::Axon, Chromosome::North);
    }

    #[test]
    #[should_panic]
    fn grow_local_bad_east() {
        let mut data = vec![Cell::new(), Cell::new()];
        let change = Page::grow_local(&mut data, 63, 0, CellType::Axon, Chromosome::East);
    }
}
