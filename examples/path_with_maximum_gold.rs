/// # Path with maximum gold
///
/// ## Problem
///
/// In a grid of size `N x N`, each cell has an integer representing the amount of gold in that cell,
/// `0` if it's empty.
///
/// Return the path where you can get the maxium amount of gold, under these conditions:
/// - You can only go to the right, bottom or diagonal to the bottom-right.
/// - You cannot visit the same cell more than once.
/// - You will start from `(1, 1)` and end to `(N, N)`.
///
/// ## Solution
///
/// Provided as an assignment from my professor in Algorithms, this is my solution.
/// It is solved as a dynamic programming (top-down version) problem:
///
/// ```
/// r(x, y) = {
///   0, if x = N or y = N,
///   f(x, y) + max{ r(x + 1, y), r(x, y + 1), r(x + 1, y + 1) }, elsewhere
/// }
/// ```
///
/// Where:
/// - `r(x, y)` is the sum of the maximum amount of gold we would get if we started from the cell `(x, y)`.
/// - `f(x, y)` the amount of gold the cell `(x, y)` has.

fn main() {
    const SIZE: usize = 5;

    const GRID: Grid<SIZE> = [
        [4, 0, 2, 0, 40],
        [0, 7, 5, 0, 0],
        [0, 0, 6, 3, 0],
        [0, 2, 1, 0, 0],
        [0, 8, 0, 8, 2],
    ];

    let (amount_of_gold, path) = path_with_maximum_gold(&GRID);
    // Convert it into a HashSet for the `O(1)` lookup performance. It's optional
    let path: std::collections::HashSet<_> = path.into_iter().collect();

    for y in 0..SIZE {
        for x in 0..SIZE {
            let cell = Cell { x, y };

            if path.contains(&cell) {
                print!("\x1b[31m");
            }

            print!("{} ", GRID[y][x]);

            if path.contains(&cell) {
                print!("\x1b[0m");
            }
        }

        println!();
    }

    println!("\nMaximum amount of gold to be collected: {amount_of_gold}");
}

type Num = u16;
type Grid<const N: usize> = [[Num; N]; N];
type MemoizationTable<const N: usize> = [[Option<(Num, Direction)>; N]; N];

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

fn path_with_maximum_gold<const N: usize>(grid: &Grid<N>) -> (Num, Vec<Cell>) {
    let starting_cell = Cell { x: 0, y: 0 };

    let mut memoization_table: MemoizationTable<N> = [[None; N]; N];
    let max_amount = path_with_maximum_gold_sol(grid, &mut memoization_table, starting_cell);

    // Find the path via the tabulation table
    let mut path: Vec<Cell> = [starting_cell].to_vec();
    let mut curr_cell = starting_cell;

    while curr_cell != (Cell { x: N - 1, y: N - 1 }) {
        curr_cell = match memoization_table[curr_cell.y][curr_cell.x] {
            Some((_, Direction::Right)) => (curr_cell.x + 1, curr_cell.y).into(),
            Some((_, Direction::Bottom)) => (curr_cell.x, curr_cell.y + 1).into(),
            Some((_, Direction::DiagonalBR)) => (curr_cell.x + 1, curr_cell.y + 1).into(),
            None => continue,
        };

        path.push(curr_cell);
    }

    (max_amount, path)
}

fn path_with_maximum_gold_sol<const N: usize>(
    grid: &Grid<N>,
    memoization_table: &mut MemoizationTable<N>,
    cell: Cell,
) -> Num {
    // If x or y are out of bounds
    if cell.x == N || cell.y == N {
        return 0;
    }

    // Return cached value if it exists
    if let Some((cached, _)) = memoization_table[cell.y][cell.x] {
        return cached;
    }

    let diagonal_br_gold = (Cell::from((cell.x + 1, cell.y + 1)), Direction::DiagonalBR);
    let bottom_gold = (Cell::from((cell.x, cell.y + 1)), Direction::Bottom);
    let right_gold = (Cell::from((cell.x + 1, cell.y)), Direction::Right);

    let (mut max_dir, mut max_gold) = (Direction::Bottom, u16::MIN);

    for (cell, direction) in [right_gold, bottom_gold, diagonal_br_gold] {
        let gold = path_with_maximum_gold_sol(grid, memoization_table, cell);

        if gold > max_gold {
            max_dir = direction;
            max_gold = gold;
        }
    }

    let amount = grid[cell.y][cell.x] + max_gold;

    // Cache and return the result
    memoization_table[cell.y][cell.x] = Some((amount, max_dir));
    amount
}
