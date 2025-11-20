use ahash::{AHashMap, AHashSet};
use smallvec::SmallVec;
use std::collections::VecDeque;

// Each position uses i8 (avoiding casting hell), hence the map cannot exceed 127×127 size.
type Pos = (i8, i8);

// Handling at most 16 boxes.
type Boxes = SmallVec<[Pos; 16]>;

const MAX_SIZE: usize = 127;
const DIRECTIONS: [(i8, i8, char); 4] = [(1, 0, 'D'), (-1, 0, 'U'), (0, 1, 'R'), (0, -1, 'L')];

#[derive(Clone)]
struct State {
    boxes: Boxes,
    player: Pos,
}

pub fn solve(level: &[&str]) -> Option<String> {
    let height = level.len();
    let width = level.iter().map(|row| row.len()).max();

    let width = width.expect("Level has no rows or all rows are empty");
    if height > MAX_SIZE || width > MAX_SIZE {
        panic!("Level too big: max size is {}x{}", MAX_SIZE, MAX_SIZE);
    }

    let mut grid: [[char; MAX_SIZE]; MAX_SIZE] = [[' '; MAX_SIZE]; MAX_SIZE];
    for r in 0..height {
        for c in 0..level[r].len() {
            grid[r][c] = level[r].as_bytes()[c] as char;
        }
    }

    let mut player = (0, 0);
    let mut boxes = Boxes::new();
    let mut goals = AHashSet::default();

    for row in 0..height {
        for col in 0..width {
            let char = grid[row][col];

            let row = row as i8;
            let col = col as i8;
            match char {
                '@' | '+' => player = (row, col),
                '$' | '*' => boxes.push((row, col)),
                _ => {}
            }
            if matches!(char, '.' | '*' | '+') {
                goals.insert((row, col));
            }
        }
    }

    // Keep boxes in a fixed order so the same setup isn't counted twice
    // e.g. [(2,3),(4,5)] and [(4,5),(2,3)] are treated the same.
    boxes.sort_unstable();
    let mut visited_boxes: AHashSet<Boxes> = AHashSet::from([boxes.clone()]);

    let init_state = State { boxes, player };
    let mut queue: VecDeque<(State, String)> = VecDeque::from([(init_state, String::new())]);

    // Using in-place mutation avoids cloning and heap allocation, making the flood fill faster.
    let mut reachable = [[false; MAX_SIZE]; MAX_SIZE];
    let mut came_from: AHashMap<Pos, (Pos, char)> = AHashMap::default();
    let mut stack = Vec::with_capacity(MAX_SIZE * MAX_SIZE);

    while let Some((state, path)) = queue.pop_front() {
        if state.boxes.iter().all(|b| goals.contains(b)) {
            return Some(path);
        }

        stack.clear();
        came_from.clear();
        for row in &mut reachable {
            row.fill(false);
        }

        flood_fill(
            state.player,
            &state.boxes,
            &grid,
            &mut reachable,
            &mut came_from,
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

                let mut new_boxes = state.boxes.clone();
                new_boxes[i] = (box_row, box_col);
                new_boxes.sort_unstable();

                if new_boxes
                    .iter()
                    .any(|&box_pos| is_dead_corner(box_pos, &goals, &grid))
                    || !visited_boxes.insert(new_boxes.clone())
                {
                    continue;
                }

                // TODO: DO NOT reconstruct here, only append box pushes, and reconstruct later
                let new_path = format!(
                    "{}{}{}",
                    path,
                    reconstruct_path(&came_from, (player_row, player_col)),
                    push_ch
                );

                queue.push_back((
                    State {
                        boxes: new_boxes,
                        player: box_position,
                    },
                    new_path,
                ));
            }
        }
    }

    None
}

fn is_free(row: i8, col: i8, boxes: &Boxes, grid: &[[char; MAX_SIZE]; MAX_SIZE]) -> bool {
    grid[row as usize][col as usize] != '#' && !boxes.contains(&(row, col))
}

fn is_dead_corner(
    box_pos: Pos,
    goals: &AHashSet<Pos>,
    grid: &[[char; MAX_SIZE]; MAX_SIZE],
) -> bool {
    if goals.contains(&box_pos) {
        return false;
    }

    let (row, col) = box_pos;
    let blocked = |r: i8, c: i8| grid[r as usize][c as usize] == '#';

    let up = blocked(row - 1, col);
    let down = blocked(row + 1, col);
    let left = blocked(row, col - 1);
    let right = blocked(row, col + 1);

    (up || down) && (left || right)
}

fn flood_fill(
    start: Pos,
    boxes: &Boxes,
    grid: &[[char; MAX_SIZE]; MAX_SIZE],
    reachable: &mut [[bool; MAX_SIZE]; MAX_SIZE],
    came_from: &mut AHashMap<Pos, (Pos, char)>,
    stack: &mut Vec<Pos>,
) {
    let dirs = [(1, 0, 'd'), (-1, 0, 'u'), (0, 1, 'r'), (0, -1, 'l')];

    stack.push(start);
    reachable[start.0 as usize][start.1 as usize] = true;
    came_from.insert(start, (start, '\0'));

    while let Some((row, col)) = stack.pop() {
        for &(dr, dc, mv) in &dirs {
            let new_row = row + dr;
            let new_col = col + dc;
            let (ur, uc) = (new_row as usize, new_col as usize);

            if is_free(new_row, new_col, boxes, grid) && !reachable[ur][uc] {
                reachable[ur][uc] = true;
                came_from.insert((new_row, new_col), ((row, col), mv));
                stack.push((new_row, new_col));
            }
        }
    }
}

fn reconstruct_path(map: &AHashMap<Pos, (Pos, char)>, end: Pos) -> String {
    let mut out = Vec::new();
    let mut cur = end;

    while let Some(&(prev, mv)) = map.get(&cur) {
        if mv == '\0' {
            break;
        }
        out.push(mv);
        cur = prev;
    }

    out.reverse();
    out.into_iter().collect()
}

fn main() {
    test_examples();
    // TODO: This map takes several minutes to solve, optimize the solver further
    // let boring1 = [
    //     "########", "#..$.$ #", "# $..  #", "# $ *$ #", "# # $. #", "#*$**$.#", "# .@  ##",
    //     "#######",
    // ];

    // // Box push only: "UUUUUUURRUDRLDLLDRUDURDRRDDUDLDLUULRRDRULLLUDURDLRRULDDLUULDLDD"
    // let expected = "llUUUUddddrrUUUdRRUrDllluRuuLDrddrruuuLrddLruulDlluRdrrddlllUdrrruullDurrddlUlldRuuulDrddlddlluuuuRRDDuullddddrrrUdllluuuurrddDrdLuuurDurrdLulldddrrUULrddlluRuululldddRluuurrurDllldddrRUrrruuLLLrrrddlllUdrrruullDurrddlUlldRuuulDrdrruuLrddlldldlluuuRRlldddrruULulDDurrrrrdLulldddrrUULulDrrruLruulDD";
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
            // "ULURDLUU"
            "dlUrrrdLullddrUluRuulDrddrruLdlUU",
        ),
    );

    levels.insert(
        "petitesse",
        (
            &["#####", "#   #", "#.$.#", "# $ #", "#+$ #", "#####"],
            // "LRLU"
            "uuurrdddLruuullddRluurrdLddrU",
        ),
    );

    levels.insert(
        "scoria",
        (
            &[
                "  ####", "  #  #", "### .#", "#  * #", "# #@ #", "# $* #", "##   #", " #####",
            ],
            // "UURDLRUUUDR"
            "UrdddlUruulllddRluurrruulDrdLrdddlluRdrUUUlDrddlluluuR",
        ),
    );

    levels.insert(
        "autogen",
        (
            &[
                "########", "###  . #", "## * # #", "## .$  #", "##  #$##", "### @ ##", "########",
                "########",
            ],
            // "RLULULURRDUL"
            "luluuRurrrddlLrruullldlddrdrrUdlluluururrrddLruullldlddrUddrruuLUdrddlluluuRuRDllddrUddrruuL",
        ),
    );

    levels.insert(
        "squared",
        (
            &[
                "#######", "# . * #", "#.*$ .#", "# $ $ #", "#*$ .*#", "#@* * #", "#######",
            ],
            // "UURURRDLLU"
            "UURURRDLdLU",
        ),
    );

    levels.insert(
        "boring2",
        (
            &[
                "#######", "#  .+.#", "#.*.####", "# $ $..#", "# $#$$ #", "#*$ $  #", "#      #",
                "########",
            ],
            // "RRDUUURRRURRDURRRDUUUUDURRLUUURLLLDLUUUURULLDUUULLL"
            "lddRRDrddlllllUUURRRllldddrrrrUdlluRldlluRluurruullDurrddlUluRRRllddDlUdddrUUUruullDurrddlUluRRldddlddrrruLdlUUUruulldRddlddrrrrruLLLdlluururrDulldlddrrrrruuLrddllllUUUUluRdddlUrddrdrruLLdlluururrDrrddllllUUUlddrrdrruLLL",
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
