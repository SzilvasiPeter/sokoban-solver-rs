use ahash::AHashSet;
use smallvec::SmallVec;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
// use std::collections::VecDeque;

// Each position uses i8 (avoiding casting hell), hence the map cannot exceed 127×127 size.
type Pos = (i8, i8);
type BoxOrGoal = SmallVec<[Pos; 15]>;
type Grid = [[char; MAX_SIZE]; MAX_SIZE];
type BoolGrid = [[bool; MAX_SIZE]; MAX_SIZE];

const MAX_SIZE: usize = 127;
const DIRECTIONS: [(i8, i8, u8); 4] = [(1, 0, b'D'), (-1, 0, b'U'), (0, 1, b'R'), (0, -1, b'L')];

#[derive(Clone, Eq, PartialEq)]
struct State {
    boxes: BoxOrGoal,
    player: Pos,
    pushes: SmallVec<[u8; 128]>,
    cost: usize,     // Number of pushes made so far
    priority: usize, // cost + heuristic
}

// We need Ord for BinaryHeap to work as a Priority Queue
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Rust's BinaryHeap is a max-heap, so we reverse the comparison
        // to get the smallest priority (min-heap behavior).
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            boxes: SmallVec::new(),
            player: (0, 0),
            pushes: SmallVec::new(),
            cost: 0,
            priority: 0,
        }
    }
}

pub fn solve(level: &[&str]) -> Option<String> {
    let height = level.len();
    let width = level.iter().map(|row| row.len()).max();

    let width = width.expect("Level has no rows or all rows are empty");
    if height > MAX_SIZE || width > MAX_SIZE {
        panic!("Level too big: max size is {}x{}", MAX_SIZE, MAX_SIZE);
    }

    let mut grid: Grid = [[' '; MAX_SIZE]; MAX_SIZE];
    // let mut state = State::default();
    let mut initial_player = (0, 0);
    let mut initial_boxes = BoxOrGoal::new();

    let mut goals = BoxOrGoal::new();

    for (r, row) in level.iter().enumerate().take(height) {
        for (c, &byte) in row.as_bytes().iter().enumerate() {
            let char = byte as char;
            grid[r][c] = char;

            let pos = (r as i8, c as i8);
            match char {
                '@' | '+' => initial_player = pos, // state.player = (irow, icol),
                '$' | '*' => initial_boxes.push(pos), // state.boxes.push((irow, icol)),
                _ => {}
            }
            if matches!(char, '.' | '*' | '+') {
                goals.push(pos);
            }
        }
    }

    // Keep boxes in a fixed order so the same setup isn't counted twice
    // e.g. [(2,3),(4,5)] and [(4,5),(2,3)] are treated the same.
    // state.boxes.sort_unstable();
    initial_boxes.sort_unstable();

    let mut dead = [[true; MAX_SIZE]; MAX_SIZE];
    dead_squares(&grid, &goals, &mut dead);

    // Using in-place mutation avoids cloning and heap allocation, making the flood fill faster.
    let mut reachable = [[false; MAX_SIZE]; MAX_SIZE];
    let mut stack = Vec::with_capacity(MAX_SIZE * MAX_SIZE);
    let mut came_from: Vec<u8> = Vec::new();

    // let mut visited_boxes: AHashSet<BoxOrGoal> = AHashSet::with_capacity(65536);
    // visited_boxes.insert(state.boxes.clone());

    // let mut queue: VecDeque<State> = VecDeque::with_capacity(256);
    // queue.insert(0, state);

    // SETUP A* SEARCH
    // Buffer 2: Temporary buffer for calculating normalized player in FUTURE states
    let mut visited: AHashSet<(BoxOrGoal, Pos)> = AHashSet::with_capacity(65536);
    let mut queue: BinaryHeap<State> = BinaryHeap::new();
    let mut normalization_buffer = [[false; MAX_SIZE]; MAX_SIZE];

    // Normalize initial state
    let norm_player = get_normalized_player(
        initial_player,
        &initial_boxes,
        &grid,
        &mut normalization_buffer,
        &mut stack,
    );
    visited.insert((initial_boxes.clone(), norm_player));

    let initial_h = heuristic(&initial_boxes, &goals);

    queue.push(State {
        boxes: initial_boxes,
        player: initial_player,
        pushes: SmallVec::new(),
        cost: 0,
        priority: initial_h,
    });

    let mut num_branch = 0;
    while let Some(state) = queue.pop() {
        num_branch += 1;
        if state.boxes.iter().all(|b| goals.contains(b)) {
            println!("num_branch: {}", num_branch);
            return Some(state.pushes.iter().map(|i| *i as char).collect::<String>());
        }

        stack.clear();
        came_from.clear();
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

                if is_locked(&new_boxes, &goals, &grid)
                    // || !visited.insert(new_boxes.clone())
                    || is_square_deadlock(new_box_row, new_box_col, &new_boxes, &goals, &grid)
                {
                    continue;
                }

                let norm_player = get_normalized_player(
                    box_position,
                    &new_boxes,
                    &grid,
                    &mut normalization_buffer,
                    &mut stack,
                );

                if !visited.insert((new_boxes.clone(), norm_player)) {
                    continue;
                }

                let mut new_pushes = state.pushes.clone();
                new_pushes.push(push_ch);

                let new_cost = state.cost + 1;
                let h = heuristic(&new_boxes, &goals);

                queue.push(State {
                    boxes: new_boxes,
                    player: box_position,
                    pushes: new_pushes,
                    cost: new_cost,
                    priority: new_cost + h,
                });
            }
        }
    }

    None
}

// 1. Heuristic: Sum of Manhattan distances from every box to its nearest goal
fn heuristic(boxes: &BoxOrGoal, goals: &BoxOrGoal) -> usize {
    let mut total = 0;
    for &(br, bc) in boxes {
        let mut min = usize::MAX;
        for &(gr, gc) in goals {
            let dist = (br - gr).abs() + (bc - gc).abs();
            if (dist as usize) < min {
                min = dist as usize;
            }
        }
        total += min;
    }
    total
}

// 2. Player Normalization
// Returns the "canonical" position for the player (e.g., smallest coordinate reachable)
// This ensures that if the player moves around in empty space without pushing, the state is identical.
fn get_normalized_player(
    pos: Pos,
    boxes: &BoxOrGoal,
    grid: &Grid,
    normalization_buffer: &mut BoolGrid,
    stack: &mut Vec<Pos>,
) -> Pos {
    // Reset buffers
    for row in normalization_buffer.iter_mut() {
        row.fill(false);
    }
    stack.clear();

    mark_reachable(pos, boxes, grid, normalization_buffer, stack);

    // Find top-left-most reachable square
    for r in 0..MAX_SIZE {
        for c in 0..MAX_SIZE {
            if normalization_buffer[r][c] {
                return (r as i8, c as i8);
            }
        }
    }
    pos // Should not happen if p is valid
}

fn is_free(row: i8, col: i8, boxes: &BoxOrGoal, grid: &Grid) -> bool {
    grid[row as usize][col as usize] != '#' && !boxes.contains(&(row, col))
}

fn is_wall(row: i8, col: i8, grid: &Grid) -> bool {
    grid[row as usize][col as usize] == '#'
}

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

// num_branch: 1_183_902
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
