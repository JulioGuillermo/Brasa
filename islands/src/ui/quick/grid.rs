//! Where the tiles sit: columns and rows per page by panel size, and which tile an arrow key leads to.

use crate::config::Size;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arrow {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub columns: usize,
    pub rows: usize,
    pub tile_count: usize,
}

impl Grid {
    /// Small is three by two, medium four by three, large every tile on one page.
    pub fn new(size: Size, tile_count: usize) -> Grid {
        let columns = match size {
            Size::Small => 3,
            Size::Medium | Size::Large => 4,
        };
        let rows = match size {
            Size::Small => 2,
            Size::Medium => 3,
            Size::Large => tile_count.div_ceil(columns).max(1),
        };
        Grid { columns, rows, tile_count }
    }

    pub fn per_page(&self) -> usize {
        self.columns * self.rows
    }

    pub fn pages(&self) -> usize {
        self.tile_count.div_ceil(self.per_page()).max(1)
    }

    /// The tile an arrow key leads to; None past the last row of the last page.
    pub fn neighbour(&self, index: usize, arrow: Arrow) -> Option<usize> {
        let page = index / self.per_page();
        let place = index % self.per_page();
        let (row, column) = (place / self.columns, place % self.columns);
        let last = self.tile_count - 1;
        match arrow {
            Arrow::Left => Some(index.saturating_sub(1)),
            Arrow::Right => Some((index + 1).min(last)),
            Arrow::Up if row > 0 => Some(index - self.columns),
            Arrow::Up if page > 0 => Some((page - 1) * self.per_page() + (self.rows - 1) * self.columns + column),
            Arrow::Up => Some(index),
            Arrow::Down if row < self.rows - 1 && index + self.columns <= last => Some(index + self.columns),
            Arrow::Down if page < self.pages() - 1 => Some(((page + 1) * self.per_page() + column).min(last)),
            Arrow::Down => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_grid_of_fourteen() -> Grid {
        Grid::new(Size::Small, 14)
    }

    #[test]
    fn sizes_decide_the_pages() {
        assert_eq!(small_grid_of_fourteen().pages(), 3);
        assert_eq!(Grid::new(Size::Medium, 14).pages(), 2);
        let large = Grid::new(Size::Large, 14);
        assert_eq!((large.rows, large.pages()), (4, 1));
    }

    #[test]
    fn left_and_right_walk_across_rows_and_pages_and_stop_at_the_ends() {
        let grid = small_grid_of_fourteen();
        assert_eq!(grid.neighbour(0, Arrow::Left), Some(0));
        assert_eq!(grid.neighbour(2, Arrow::Right), Some(3));
        assert_eq!(grid.neighbour(5, Arrow::Right), Some(6));
        assert_eq!(grid.neighbour(13, Arrow::Right), Some(13));
    }

    #[test]
    fn down_moves_a_row_then_a_page_then_leaves_the_tiles() {
        let grid = small_grid_of_fourteen();
        assert_eq!(grid.neighbour(1, Arrow::Down), Some(4));
        assert_eq!(grid.neighbour(4, Arrow::Down), Some(7));
        assert_eq!(grid.neighbour(10, Arrow::Down), Some(13));
        assert_eq!(grid.neighbour(13, Arrow::Down), None);
        assert_eq!(grid.neighbour(12, Arrow::Down), None);
    }

    #[test]
    fn down_on_a_short_last_row_goes_to_the_last_tile_of_the_next_page() {
        let grid = Grid::new(Size::Small, 8);
        assert_eq!(grid.neighbour(5, Arrow::Down), Some(7));
    }

    #[test]
    fn up_moves_a_row_then_a_page_then_stays() {
        let grid = small_grid_of_fourteen();
        assert_eq!(grid.neighbour(4, Arrow::Up), Some(1));
        assert_eq!(grid.neighbour(7, Arrow::Up), Some(4));
        assert_eq!(grid.neighbour(6, Arrow::Up), Some(3));
        assert_eq!(grid.neighbour(1, Arrow::Up), Some(1));
    }
}
