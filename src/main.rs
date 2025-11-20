use ahash::{AHashMap as HashMap, AHashSet as HashSet};
use smallvec::SmallVec;
use std::collections::VecDeque;

type Pos = (usize, usize);
type Boxes = SmallVec<[Pos; 10]>;

#[derive(Clone)]
struct State {
    boxes: Boxes,
    player: Pos,
}

pub fn solve(level: &[&str]) -> Option<String> {
    let height = level.len();
    let width = level.iter().map(|r| r.len()).max().unwrap_or(0);

    let grid: Vec<Vec<char>> = level
        .iter()
        .map(|row| {
            let mut v: Vec<char> = row.chars().collect();
            v.resize(width, ' ');
            v
        })
        .collect();

    let mut box_positions = Boxes::new();
    let mut goals = HashSet::default();
    let mut player_pos = (0, 0);

    // Parse map
    for r in 0..height {
        for c in 0..width {
            let ch = grid[r][c];
            match ch {
                '@' | '+' => player_pos = (r, c),
                '$' | '*' => box_positions.push((r, c)),
                _ => {}
            }
            if matches!(ch, '.' | '*' | '+') {
                goals.insert((r, c));
            }
        }
    }

    box_positions.sort_unstable();

    let mut visited_boxes: HashSet<Boxes> = HashSet::default();

    let init_state = State {
        boxes: box_positions.clone(),
        player: player_pos,
    };
    visited_boxes.insert(box_positions.clone());

    let mut queue = VecDeque::new();
    queue.push_back((init_state, String::new()));

    let mut reachable = vec![vec![false; width]; height];
    let mut came_from: HashMap<Pos, (Pos, char)> = HashMap::default();
    let mut stack = Vec::new();

    let directions = [(1, 0, 'D'), (-1, 0, 'U'), (0, 1, 'R'), (0, -1, 'L')];

    while let Some((state, path)) = queue.pop_front() {
        if state.boxes.iter().all(|b| goals.contains(b)) {
            return Some(path);
        }

        for row in &mut reachable {
            row.fill(false);
        }
        came_from.clear();
        stack.clear();

        flood_fill(
            state.player,
            &state.boxes,
            &grid,
            &mut reachable,
            &mut came_from,
            &mut stack,
        );

        for (i, &b) in state.boxes.iter().enumerate() {
            let (br, bc) = b;

            for &(dr, dc, push_ch) in &directions {
                // required player position
                let pr = br as isize - dr;
                let pc = bc as isize - dc;
                if pr < 0 || pc < 0 {
                    continue;
                }
                let player_needed = (pr as usize, pc as usize);
                if !reachable[player_needed.0][player_needed.1] {
                    continue;
                }

                // new box position
                let nr = br as isize + dr;
                let nc = bc as isize + dc;
                if !is_free(nr, nc, &state.boxes, &grid) {
                    continue;
                }
                let new_box_pos = (nr as usize, nc as usize);

                let mut new_boxes = state.boxes.clone();
                new_boxes[i] = new_box_pos;
                new_boxes.sort_unstable();

                if new_boxes
                    .iter()
                    .any(|&bx| is_dead_corner(bx, &goals, &grid))
                {
                    continue;
                }

                // Check visited *box configuration* only
                if !visited_boxes.insert(new_boxes.clone()) {
                    continue;
                }

                let moves = reconstruct_path(&came_from, player_needed);

                let mut new_path = path.clone();
                new_path.push_str(&moves);
                new_path.push(push_ch);

                queue.push_back((
                    State {
                        boxes: new_boxes,
                        player: b,
                    },
                    new_path,
                ));
            }
        }
    }

    None
}

fn flood_fill(
    start: Pos,
    boxes: &Boxes,
    grid: &[Vec<char>],
    reachable: &mut [Vec<bool>],
    came_from: &mut HashMap<Pos, (Pos, char)>,
    stack: &mut Vec<Pos>,
) {
    reachable[start.0][start.1] = true;
    came_from.insert(start, (start, '\0'));
    stack.push(start);

    let dirs = [(1, 0, 'd'), (-1, 0, 'u'), (0, 1, 'r'), (0, -1, 'l')];

    while let Some((r, c)) = stack.pop() {
        for &(dr, dc, mv) in &dirs {
            let nr = r as isize + dr;
            let nc = c as isize + dc;

            let (ur, uc) = (nr as usize, nc as usize);

            if is_free(nr, nc, boxes, grid) && !reachable[ur][uc] {
                reachable[ur][uc] = true;
                came_from.insert((ur, uc), ((r, c), mv));
                stack.push((ur, uc));
            }
        }
    }
}

fn reconstruct_path(map: &HashMap<Pos, (Pos, char)>, end: Pos) -> String {
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

#[inline]
fn is_free(row: isize, col: isize, boxes: &Boxes, grid: &[Vec<char>]) -> bool {
    if row < 0 || col < 0 {
        return false;
    }
    let (r, c) = (row as usize, col as usize);
    r < grid.len() && c < grid[0].len() && grid[r][c] != '#' && !boxes.contains(&(r, c))
}

fn is_dead_corner(b: Pos, goals: &HashSet<Pos>, grid: &[Vec<char>]) -> bool {
    if goals.contains(&b) {
        return false;
    }

    let (r, c) = b;
    let h = grid.len();
    let w = grid[0].len();

    let wall = |rr: isize, cc: isize| {
        if rr < 0 || cc < 0 {
            return false;
        }
        let rr = rr as usize;
        let cc = cc as usize;
        rr < h && cc < w && grid[rr][cc] == '#'
    };

    let up = wall(r as isize - 1, c as isize);
    let down = wall(r as isize + 1, c as isize);
    let left = wall(r as isize, c as isize - 1);
    let right = wall(r as isize, c as isize + 1);

    (up || down) && (left || right)
}

fn main() {
    test_examples();
    // let level = [
    //     "########", "#..$.$ #", "# $..  #", "# $ *$ #", "# # $. #", "#*$**$.#", "# .@  ##",
    //     "#######",
    // ];
    // let solution = solve(&level);
    // println!("{:?}", solution);
}

fn test_examples() {
    use std::collections::HashMap;
    let mut levels: HashMap<&str, (&[&str], &str)> = HashMap::new();

    levels.insert(
        "microban",
        (
            &[
                "####", "# .#", "#  ###", "#*@  #", "#  $ #", "#  ###", "####",
            ],
            "dlUrrrdLullddrUluRuulDrddrruLdlUU",
        ),
    );

    levels.insert(
        "petitesse",
        (
            &["#####", "#   #", "#.$.#", "# $ #", "#+$ #", "#####"],
            "uuurrdddLruuullddRluurrdLddrU",
        ),
    );

    levels.insert(
        "scoria",
        (
            &[
                "  ####", "  #  #", "### .#", "#  * #", "# #@ #", "# $* #", "##   #", " #####",
            ],
            "UrdddlUruulllddRluurrruulDrdLrdddlluRdrUUUlDrddlluluuR",
        ),
    );

    levels.insert(
        "autogen",
        (
            &[
                "########",
                "###  . #",
                "## * # #",
                "## .$  #",
                "##  #$##",
                "### @ ##",
                "########",
                "########",
            ],
            "luluuRurrrddlLrruullldlddrdrrUdlluluururrrddLruullldlddrUddrruuLUdrddlluluuRuRDllddrUddrruuL",
        ),
    );

    levels.insert(
        "squared",
        (
            &[
                "#######", "# . * #", "#.*$ .#", "# $ $ #", "#*$ .*#", "#@* * #", "#######",
            ],
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
