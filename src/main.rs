use ahash::AHashSet;
use smallvec::SmallVec;
use std::collections::VecDeque;

// Each position uses i8 (avoiding casting hell), hence the map cannot exceed 127×127 size.
type Pos = (i8, i8);
type BoxOrGoal = SmallVec<[Pos; 15]>;
type Path = SmallVec<[u8; 64]>;
type Grid = [[char; MAX_SIZE]; MAX_SIZE];
type BoolGrid = [[bool; MAX_SIZE]; MAX_SIZE];

const MAX_SIZE: usize = 127;
const DIRECTIONS: [(i8, i8, u8); 4] = [(1, 0, b'D'), (-1, 0, b'U'), (0, 1, b'R'), (0, -1, b'L')];

#[derive(Clone)]
struct State {
    boxes: BoxOrGoal,
    player: Pos,
    path: Path,
}

impl Default for State {
    fn default() -> Self {
        Self {
            boxes: BoxOrGoal::new(),
            player: (0, 0),
            path: Path::new(),
        }
    }
}

fn solve(level: &[&str]) -> Option<String> {
    let height = level.len();
    let width = level.iter().map(|row| row.len()).max();

    let width = width.expect("Level has no rows or all rows are empty");
    if height > MAX_SIZE || width > MAX_SIZE {
        panic!("Level too big: max size is {}x{}", MAX_SIZE, MAX_SIZE);
    }

    let mut grid: Grid = [[' '; MAX_SIZE]; MAX_SIZE];
    let mut state = State::default();
    let mut goals = BoxOrGoal::new();

    for (r, row) in level.iter().enumerate().take(height) {
        for (c, &byte) in row.as_bytes().iter().enumerate() {
            let char = byte as char;
            grid[r][c] = char;

            let irow = r as i8;
            let icol = c as i8;
            match char {
                '@' | '+' => state.player = (irow, icol),
                '$' | '*' => state.boxes.push((irow, icol)),
                _ => {}
            }
            if matches!(char, '.' | '*' | '+') {
                goals.push((irow, icol));
            }
        }
    }

    // Keep boxes in a fixed order so the same setup isn't counted twice
    // e.g. [(2,3),(4,5)] and [(4,5),(2,3)] are treated the same.
    state.boxes.sort_unstable();

    let mut visited_boxes: AHashSet<BoxOrGoal> = AHashSet::with_capacity(65536);
    visited_boxes.insert(state.boxes.clone());

    let mut queue: VecDeque<State> = VecDeque::with_capacity(256);
    queue.insert(0, state);

    let mut dead = [[true; MAX_SIZE]; MAX_SIZE];
    dead_squares(&grid, &goals, &mut dead);

    // Using in-place mutation avoids cloning and heap allocation, making the flood fill faster.
    let mut reachable = [[false; MAX_SIZE]; MAX_SIZE];
    let mut stack = Vec::with_capacity(MAX_SIZE * MAX_SIZE);

    let mut num_branch = 0;
    while let Some(state) = queue.pop_front() {
        num_branch += 1;
        if state.boxes.iter().all(|b| goals.contains(b)) {
            println!("num_branch: {}", num_branch);
            return Some(state.path.iter().map(|i| *i as char).collect::<String>());
        }

        stack.clear();
        for row in &mut reachable {
            row.fill(false);
        }

        mark_reachable(
            state.player,
            &state.boxes,
            &grid,
            &mut reachable,
            &mut stack,
        );

        for (i, &box_position) in state.boxes.iter().enumerate() {
            let (box_row, box_col) = box_position;
            for &(dr, dc, push_ch) in &DIRECTIONS {
                let (new_box_row, new_box_col) = (box_row + dr, box_col + dc);
                let (new_player_row, new_player_col) = (box_row - dr, box_col - dc);

                if !reachable[new_player_row as usize][new_player_col as usize]
                    || dead[new_box_row as usize][new_box_col as usize]
                    || !is_free(new_box_row, new_box_col, &state.boxes, &grid)
                {
                    continue;
                }

                // Cloning `SmallVec` is very cheap.
                let mut new_boxes = state.boxes.clone();
                new_boxes[i] = (new_box_row, new_box_col);
                new_boxes.sort_unstable();

                if !visited_boxes.insert(new_boxes.clone())
                    || is_locked(&new_boxes, &goals, &grid)
                    || is_square_deadlock(new_box_row, new_box_col, &new_boxes, &goals, &grid)
                {
                    continue;
                }

                let mut new_path = state.path.clone();
                new_path.push(push_ch);

                queue.push_back(State {
                    boxes: new_boxes,
                    player: box_position,
                    path: new_path,
                });
            }
        }
    }

    None
}

fn is_free(row: i8, col: i8, boxes: &BoxOrGoal, grid: &Grid) -> bool {
    grid[row as usize][col as usize] != '#' && !boxes.contains(&(row, col))
}

fn is_wall(row: i8, col: i8, grid: &Grid) -> bool {
    grid[row as usize][col as usize] == '#'
}

// num_branch: 25_066_728
fn dead_squares(grid: &Grid, goals: &BoxOrGoal, dead: &mut BoolGrid) {
    let height = grid.len();
    let mut alive = AHashSet::new();
    let mut queue = Vec::new();

    for (x, y) in goals {
        alive.insert((*x, *y));
        queue.push((*x, *y));
    }

    while let Some((row, col)) = queue.pop() {
        for (dr, dc, _) in DIRECTIONS {
            let (prev_row, prev_col) = (row - dr, col - dc);
            let (player_row, player_col) = (row - 2 * dr, col - 2 * dc);

            let is_out_of_bounds = |r: i8, c: i8, h: usize, g: &Grid| {
                r < 0 || r as usize >= h || c < 0 || c as usize >= g[r as usize].len()
            };

            if is_out_of_bounds(prev_row, prev_col, height, grid)
                || is_out_of_bounds(player_row, player_col, height, grid)
            {
                continue;
            }

            if !is_wall(prev_row, prev_col, grid)
                && !is_wall(player_row, player_col, grid)
                && alive.insert((prev_row, prev_col))
            {
                queue.push((prev_row, prev_col));
            }
        }
    }

    for (y, x) in alive.iter() {
        dead[*y as usize][*x as usize] = false;
    }
}

// num_branch: 18_605_395
fn is_locked(boxes: &BoxOrGoal, goals: &BoxOrGoal, grid: &Grid) -> bool {
    let is_blocked = |r, c| !is_free(r, c, boxes, grid);
    boxes.iter().filter(|&b| !goals.contains(b)).any(|&(r, c)| {
        let up = is_wall(r - 1, c, grid);
        let down = is_wall(r + 1, c, grid);
        let left = is_wall(r, c - 1, grid);
        let right = is_wall(r, c + 1, grid);
        let h_block = is_blocked(r, c - 1) || is_blocked(r, c + 1);
        let v_block = is_blocked(r - 1, c) || is_blocked(r + 1, c);

        (up && down && h_block) || (left && right && v_block)
    })
}

// num_branch: 1183902
fn is_square_deadlock(
    box_row: i8,
    box_col: i8,
    boxes: &BoxOrGoal,
    goals: &BoxOrGoal,
    grid: &Grid,
) -> bool {
    let quadrants = [
        (-1, -1), // Checks top-left quadrant
        (-1, 1),  // Checks top-right quadrant
        (1, -1),  // Checks bottom-left quadrant
        (1, 1),   // Checks bottom-right quadrant
    ];

    for (dr, dc) in quadrants {
        // Check the other 3 corners of this specific 2x2 quadrant: adjacent vertical, horizontal, diagonal.
        if is_free(box_row + dr, box_col, boxes, grid)
            || is_free(box_row, box_col + dc, boxes, grid)
            || is_free(box_row + dr, box_col + dc, boxes, grid)
        {
            continue;
        }

        // We check if *any* of the 4 positions in this 2x2 is a box strictly outside a goal.
        let is_dead_quadrant = [
            (box_row, box_col),
            (box_row + dr, box_col),
            (box_row, box_col + dc),
            (box_row + dr, box_col + dc),
        ]
        .iter()
        .any(|p| boxes.contains(p) && !goals.contains(p));

        if is_dead_quadrant {
            return true;
        }
    }

    false
}

fn mark_reachable(
    start: Pos,
    boxes: &BoxOrGoal,
    grid: &Grid,
    reachable: &mut BoolGrid,
    stack: &mut Vec<Pos>,
) {
    stack.push(start);
    reachable[start.0 as usize][start.1 as usize] = true;

    while let Some((row, col)) = stack.pop() {
        for &(dr, dc, _) in &DIRECTIONS {
            let new_row = row + dr;
            let new_col = col + dc;
            let (ur, uc) = (new_row as usize, new_col as usize);

            if is_free(new_row, new_col, boxes, grid) && !reachable[ur][uc] {
                reachable[ur][uc] = true;
                stack.push((new_row, new_col));
            }
        }
    }
}

fn main() {
    let boring1 = &[
        "########", "#..$.$ #", "# $..  #", "# $ *$ #", "# # $. #", "#*$**$.#", "# .@  ##",
        "#######",
    ];
    // "llUUUUddddrrUUUdRRUrDllluRuuLDrddrruuuLrddLruulDlluRdrrddlllUdrrruullDurrddlUlldRuuulDrddlddlluuuuRRDDuullddddrrrUdllluuuurrddDrdLuuurDurrdLulldddrrUULrddlluRuululldddRluuurrurDllldddrRUrrruuLLLrrrddlllUdrrruullDurrddlUlldRuuulDrdrruuLrddlldldlluuuRRlldddrruULulDDurrrrrdLulldddrrUULulDrrruLruulDD"            "RRDUUURRRURRDURRRDUUUUDURRLUUURLLLDLUUUURULLDUUULLL",
    let expected = "UUUUUUURRUDRLDLLDRUDURDRRDDUDLDLUULRRDRULLLUDURDLRRULDDLUULDLDD";

    let actual = solve(boring1).expect("No solution was found!");
    assert_eq!(actual, expected);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microban() {
        let level = &[
            "####", "# .#", "#  ###", "#*@  #", "#  $ #", "#  ###", "####",
        ];
        // "dlUrrrdLullddrUluRuulDrddrruLdlUU",
        let expected = "ULURDLUU";

        let actual = solve(level).expect("No solution found");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_petitesse() {
        let level = &["#####", "#   #", "#.$.#", "# $ #", "#+$ #", "#####"];
        // "uuurrdddLruuullddRluurrdLddrU"
        let expected = "LRLU";

        let actual = solve(level).expect("No solution found");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_scoria() {
        let level = &[
            "  ####", "  #  #", "### .#", "#  * #", "# #@ #", "# $* #", "##   #", " #####",
        ];
        // "UrdddlUruulllddRluurrruulDrdLrdddlluRdrUUUlDrddlluluuR"
        let expected = "UURDLRUUUDR";

        let actual = solve(level).expect("No solution found");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_autogen() {
        let level = &[
            "########", "###  . #", "## * # #", "## .$  #", "##  #$##", "### @ ##", "########",
            "########",
        ];
        // "luluuRurrrddlLrruullldlddrdrrUdlluluururrrddLruullldlddrUddrruuLUdrddlluluuRuRDllddrUddrruuL"
        let expected = "RLULULURRDUL";

        let actual = solve(level).expect("No solution found");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_squared() {
        let level = &[
            "#######", "# . * #", "#.*$ .#", "# $ $ #", "#*$ .*#", "#@* * #", "#######",
        ];
        // "UURURRDLdLU"
        let expected = "UURURRDLLU";

        let actual = solve(level).expect("No solution found");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_boring2() {
        let level = &[
            "#######", "#  .+.#", "#.*.####", "# $ $..#", "# $#$$ #", "#*$ $  #", "#      #",
            "########",
        ];
        // "lddRRDrddlllllUUURRRllldddrrrrUdlluRldlluRluurruullDurrddlUluRRRllddDlUdddrUUUruullDurrddlUluRRldddlddrrruLdlUUUruulldRddlddrrrrruLLLdlluururrDulldlddrrrrruuLrddllllUUUUluRdddlUrddrdrruLLdlluururrDrrddllllUUUlddrrdrruLLL"
        let expected = "RRDUUURRRURRDURRRDUUUUDURRLUUURLLLDLUUUURULLDUUULLL";

        let actual = solve(level).expect("No solution found");
        assert_eq!(actual, expected);
    }
}
