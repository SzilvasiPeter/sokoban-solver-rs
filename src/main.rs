use ahash::{AHashMap, AHashSet};
use smallvec::SmallVec;
use std::collections::VecDeque;

// Each position uses i8 (avoiding casting hell), hence the map cannot exceed 127×127 size.
type Pos = (i8, i8);
type BoxOrGoal = SmallVec<[Pos; 15]>;
type Path = SmallVec<[u8; 64]>;
type Grid = [[char; MAX_SIZE]; MAX_SIZE];

const MAX_SIZE: usize = 127;
const DIRECTIONS: [(i8, i8, u8); 4] = [
    (1, 0, 'D' as u8),
    (-1, 0, 'U' as u8),
    (0, 1, 'R' as u8),
    (0, -1, 'L' as u8),
];

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
                let player_row = box_row - dr;
                let player_col = box_col - dc;
                let box_row = box_row + dr;
                let box_col = box_col + dc;

                if !reachable[player_row as usize][player_col as usize]
                    || !is_free(box_row, box_col, &state.boxes, &grid)
                {
                    continue;
                }

                // Cloning `SmallVec` is very cheap.
                let mut new_boxes = state.boxes.clone();
                new_boxes[i] = (box_row, box_col);
                new_boxes.sort_unstable();

                if !visited_boxes.insert(new_boxes.clone())
                    || is_cornered(&new_boxes, &goals, &grid)
                    || is_locked(&new_boxes, &goals, &grid)
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
fn is_cornered(boxes: &BoxOrGoal, goals: &BoxOrGoal, grid: &Grid) -> bool {
    boxes
        .iter()
        .filter(|&b| !goals.contains(b))
        .any(|&(row, col)| {
            let up = is_wall(row - 1, col, grid);
            let down = is_wall(row + 1, col, grid);
            let left = is_wall(row, col - 1, grid);
            let right = is_wall(row, col + 1, grid);

            (up || down) && (left || right)
        })
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

fn mark_reachable(
    start: Pos,
    boxes: &BoxOrGoal,
    grid: &Grid,
    reachable: &mut [[bool; MAX_SIZE]; MAX_SIZE],
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
    test_examples();
    // TODO: This map takes 1.5 minutes to solve, optimize the solver further
    // stats:
    //      num_branch: 25_066_728
    //      max_queue: 2_327_406
    //      visited_boxes.len(): 47_684_603
    //      path.len(): 63
    // let boring1 = [
    //     "########", "#..$.$ #", "# $..  #", "# $ *$ #", "# # $. #", "#*$**$.#", "# .@  ##",
    //     "#######",
    // ];

    // // "llUUUUddddrrUUUdRRUrDllluRuuLDrddrruuuLrddLruulDlluRdrrddlllUdrrruullDurrddlUlldRuuulDrddlddlluuuuRRDDuullddddrrrUdllluuuurrddDrdLuuurDurrdLulldddrrUULrddlluRuululldddRluuurrurDllldddrRUrrruuLLLrrrddlllUdrrruullDurrddlUlldRuuulDrdrruuLrddlldldlluuuRRlldddrruULulDDurrrrrdLulldddrrUULulDrrruLruulDD"
    // let expected = "UUUUUUURRUDRLDLLDRUDURDRRDDUDLDLUULRRDRULLLUDURDLRRULDDLUULDLDD";
    // let actual = solve(&boring1).expect("No solution found");
    // assert_eq!(actual, expected, "Unexpected solution");
}

fn test_examples() {
    let mut levels: AHashMap<&str, (&[&str], &str)> = AHashMap::new();

    levels.insert(
        "microban",
        (
            &[
                "####", "# .#", "#  ###", "#*@  #", "#  $ #", "#  ###", "####",
            ],
            // "dlUrrrdLullddrUluRuulDrddrruLdlUU",
            "ULURDLUU",
        ),
    );

    levels.insert(
        "petitesse",
        (
            &["#####", "#   #", "#.$.#", "# $ #", "#+$ #", "#####"],
            // "uuurrdddLruuullddRluurrdLddrU",
            "LRLU",
        ),
    );

    levels.insert(
        "scoria",
        (
            &[
                "  ####", "  #  #", "### .#", "#  * #", "# #@ #", "# $* #", "##   #", " #####",
            ],
            // "UrdddlUruulllddRluurrruulDrdLrdddlluRdrUUUlDrddlluluuR",
            "UURDLRUUUDR",
        ),
    );

    levels.insert(
        "autogen",
        (
            &[
                "########", "###  . #", "## * # #", "## .$  #", "##  #$##", "### @ ##", "########",
                "########",
            ],
            // "luluuRurrrddlLrruullldlddrdrrUdlluluururrrddLruullldlddrUddrruuLUdrddlluluuRuRDllddrUddrruuL",
            "RLULULURRDUL",
        ),
    );

    levels.insert(
        "squared",
        (
            &[
                "#######", "# . * #", "#.*$ .#", "# $ $ #", "#*$ .*#", "#@* * #", "#######",
            ],
            // "UURURRDLdLU",
            "UURURRDLLU",
        ),
    );

    levels.insert(
        "boring2",
        (
            &[
                "#######", "#  .+.#", "#.*.####", "# $ $..#", "# $#$$ #", "#*$ $  #", "#      #",
                "########",
            ],
            // "lddRRDrddlllllUUURRRllldddrrrrUdlluRldlluRluurruullDurrddlUluRRRllddDlUdddrUUUruullDurrddlUluRRldddlddrrruLdlUUUruulldRddlddrrrrruLLLdlluururrDulldlddrrrrruuLrddllllUUUUluRdddlUrddrdrruLLdlluururrDrrddllllUUUlddrrdrruLLL",
            "RRDUUURRRURRDURRRDUUUUDURRLUUURLLLDLUUUURULLDUUULLL",
        ),
    );

    for (key, (map, expected)) in levels.iter() {
        let actual = solve(map);

        match actual {
            Some(sol) if sol == *expected => {
                println!("Correct solution: {}", key);
            }

            Some(sol) => {
                println!("Unexpected solution for {}", key);

                println!("{}", sol);
            }

            None => {
                println!("No solution found for {}", key);
            }
        }
    }
}
