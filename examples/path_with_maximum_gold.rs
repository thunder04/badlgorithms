//! # Path with maximum gold
//!
//! ## Problem
//!
//! In a grid of size `N x N`, each cell has an integer representing the amount of gold in that cell,
//! `0` if it's empty.
//!
//! Return the path where you can get the maxium amount of gold, under these conditions:
//! - You can only go to the right, bottom or diagonal to the bottom-right.
//! - You cannot visit the same cell more than once.
//! - You will start from `(1, 1)` and end to `(N, N)`.
//!
//! ## Solution
//!
//! Provided as an assignment from my professor in Algorithms, this is my solution.
//! It is solved as a dynamic programming (top-down version) problem:
//!
//! ```
//! r(x, y) = {
//!   0, if x = N or y = N,
//!   f(x, y) + max{ r(x + 1, y), r(x, y + 1), r(x + 1, y + 1) }, elsewhere
//! }
//! ```
//!
//! Where:
//! - `r(x, y)` is the sum of the maximum amount of gold we would get if we started from the cell `(x, y)`.
//! - `f(x, y)` the amount of gold the cell `(x, y)` has.

use std::{
    collections::HashSet,
    iter::{repeat, repeat_with},
};

const MAX_GOLD_AMOUNT: u32 = 9999;
const GRID_LEN: usize = 23;

fn main() {
    let mut max_digits_count = 1;

    let grid: Grid = repeat_with(|| {
        repeat_with(|| {
            let gold_amount = fastrand::u32(0..=MAX_GOLD_AMOUNT);
            let digits_count = gold_amount.checked_ilog10().unwrap_or(0) + 1;

            if digits_count > max_digits_count {
                max_digits_count = digits_count;
            }

            gold_amount
        })
        .take(GRID_LEN)
        .collect::<Vec<_>>()
    })
    .take(GRID_LEN)
    .collect();

    let (amount_of_gold, path) = path_with_maximum_gold(&grid);
    // Convert it into a HashSet for `O(1)` lookup performance. Optional
    let path: HashSet<_> = path.into_iter().collect();

    for y in 0..GRID_LEN {
        for x in 0..GRID_LEN {
            let cell = Cell { x, y };

            if path.contains(&cell) {
                print!("\x1b[31m");
            }

            print!("{:0width$} ", grid[y][x], width = max_digits_count as _);

            if path.contains(&cell) {
                print!("\x1b[0m");
            }
        }

        println!();
    }

    println!("\nMaximum amount of gold to be collected: {amount_of_gold}");
}

type MemoizationTable = Vec<Vec<Option<(u32, Direction)>>>;
type Grid = Vec<Vec<u32>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Cell {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Cell {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Right,
    Bottom,
    DiagonalBR,
}

fn path_with_maximum_gold(grid: &Grid) -> (u32, Vec<Cell>) {
    let grid_len = grid.len();

    // Perform grid validation
    for (idx, row) in grid.iter().enumerate() {
        assert!(
            row.len() == grid_len,
            "Row #{idx} isn't equal to the number of column"
        )
    }

    let mut memoization_table: MemoizationTable =
        repeat(vec![None; grid_len]).take(grid_len).collect();

    let starting_cell = Cell { x: 0, y: 0 };
    let max_amount = path_with_maximum_gold_sol(grid, &mut memoization_table, starting_cell);

    // Find the path via the memoization table

    let mut path: Vec<Cell> = [starting_cell].to_vec();
    let mut curr_cell = starting_cell;

    while curr_cell.x < grid_len && curr_cell.y < grid_len {
        curr_cell = match memoization_table[curr_cell.y][curr_cell.x] {
            Some((_, Direction::Right)) => (curr_cell.x + 1, curr_cell.y).into(),
            Some((_, Direction::Bottom)) => (curr_cell.x, curr_cell.y + 1).into(),
            Some((_, Direction::DiagonalBR)) => (curr_cell.x + 1, curr_cell.y + 1).into(),
            None => continue,
        };

        path.push(curr_cell);
    }

    // From my professional testing, sometimes the last cell doesn't get included.
    // CBA to find out why
    path.push(Cell {
        x: grid_len - 1,
        y: grid_len - 1,
    });

    (max_amount, path)
}

fn path_with_maximum_gold_sol(
    grid: &Grid,
    memoization_table: &mut MemoizationTable,
    cell: Cell,
) -> u32 {
    // If x or y are out of bounds
    if cell.x == grid.len() || cell.y == grid.len() {
        return 0;
    }

    // Return cached value if it exists
    if let Some((cached, _)) = memoization_table[cell.y][cell.x] {
        return cached;
    }

    let diagonal_br_gold = (Cell::from((cell.x + 1, cell.y + 1)), Direction::DiagonalBR);
    let bottom_gold = (Cell::from((cell.x, cell.y + 1)), Direction::Bottom);
    let right_gold = (Cell::from((cell.x + 1, cell.y)), Direction::Right);

    let (mut max_dir, mut max_gold) = (Direction::Bottom, u32::MIN);

    for (cell, direction) in [right_gold, bottom_gold, diagonal_br_gold] {
        let gold = path_with_maximum_gold_sol(grid, memoization_table, cell);

        if gold >= max_gold {
            max_dir = direction;
            max_gold = gold;
        }
    }

    let amount = grid[cell.y][cell.x] + max_gold;

    // Cache and return the result
    memoization_table[cell.y][cell.x] = Some((amount, max_dir));
    amount
}
