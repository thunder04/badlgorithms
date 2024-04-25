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
/// It is solved as a dynamic programming (bottom-up version) problem:
///
/// ```
/// r(x, y) = {
///   0, if x = 0 or y = 0,
///   f(x, y) + max{ r(x - 1, y), r(x, y - 1), r(x - 1, y - 1) }, elsewhere
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
type TabulationTable<const N: usize> = [[Option<(Num, Direction)>; N]; N];
type GoldPositionCanditate = (Cell<isize>, Direction);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Cell<T> {
    pub x: T,
    pub y: T,
}

impl<T> From<(T, T)> for Cell<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Left,
    Top,
    DiagonalTL,
}

fn path_with_maximum_gold<const N: usize>(grid: &Grid<N>) -> (Num, Vec<Cell<usize>>) {
    let ending_cell: Cell<isize> = ((N - 1) as _, (N - 1) as _).into();

    let mut tabulation_table: TabulationTable<N> = [[None; N]; N];
    let max_amount = path_with_maximum_gold_sol(grid, &mut tabulation_table, ending_cell);

    // Find the path via the tabulation table
    let mut path: Vec<Cell<usize>> = [((N - 1) as _, (N - 1) as _).into()].to_vec();
    let mut curr_cell = path[0];

    while curr_cell != (Cell { x: 0, y: 0 }) {
        curr_cell = match tabulation_table[curr_cell.y][curr_cell.x] {
            Some((_, Direction::Left)) => (curr_cell.x - 1, curr_cell.y).into(),
            Some((_, Direction::Top)) => (curr_cell.x, curr_cell.y - 1).into(),
            Some((_, Direction::DiagonalTL)) => (curr_cell.x - 1, curr_cell.y - 1).into(),
            None => continue,
        };

        path.push(curr_cell);
    }

    (max_amount, path)
}

fn path_with_maximum_gold_sol<const N: usize>(
    grid: &Grid<N>,
    tabulation_table: &mut TabulationTable<N>,
    cell: Cell<isize>,
) -> Num {
    // If x or y are out of bounds
    if cell.x < 0 || cell.y < 0 {
        return 0;
    }

    let (cell_x, cell_y) = (cell.x as usize, cell.y as usize);

    // Return cached value if it exists
    if let Some((cached, _)) = tabulation_table[cell_y][cell_x] {
        return cached;
    }

    let (mut max_dir, mut max_gold) = (Direction::Left, u16::MIN);

    // Solving it bottom-up also means to mirror diagonally the moving constraint,
    // so go left, up and diagonal up-left.

    let top_gold: GoldPositionCanditate = ((cell.x, cell.y - 1).into(), Direction::Top);
    let left_gold: GoldPositionCanditate = ((cell.x - 1, cell.y).into(), Direction::Left);
    let diagonal_tl_gold: GoldPositionCanditate =
        ((cell.x - 1, cell.y - 1).into(), Direction::DiagonalTL);

    for (cell, direction) in [left_gold, top_gold, diagonal_tl_gold] {
        let gold = path_with_maximum_gold_sol(grid, tabulation_table, cell);

        if gold > max_gold {
            max_dir = direction;
            max_gold = gold;
        }
    }

    let amount = grid[cell_y][cell_x] + max_gold;

    // Cache and return the result
    tabulation_table[cell_y][cell_x] = Some((amount, max_dir));
    amount
}
