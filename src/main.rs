use sokoban_solver::solve;

fn main() {
    // Takes around 25 seconds to solve
    let boring1 = &[
        "########", "#..$.$ #", "# $..  #", "# $ *$ #", "# # $. #", "#*$**$.#", "# .@  ##",
        "#######",
    ];
    let expected = "UURRUrDlllddllUUUdddrruuURuuLDrddrruLruuLDlluRdrrddlllUdrrruullDurrddlUlldRlddlluuuUddddrruuuruulDrddlddlluuuuRRDullddddrrrUdllluuuurrdDDrdLuuurDurrdLulldddrrUdlluuurrruuLrddllldddrruULrddlluRuululldddRRUrrruLruulDLLuRdrrddlllUdrrruullDurrddlUlldRuuulDrddldlluuuRRlldddrruULulDrrrrruuLrdddLulldddrrUdlluuullDurrdddrruULulDrrruLruulDD";

    let actual = solve(boring1).expect("No solution was found!");
    assert_eq!(actual, expected);
}

// --- Run solver for a map ---
// ----------------------------
// use serde::Deserialize;

// #[derive(Deserialize)]
// struct Level {
//     lines: Vec<String>,
// }

// #[derive(Deserialize)]
// struct LevelFile {
//     levels: Vec<Level>,
//     name: String,
// }

// fn main() -> std::io::Result<()> {
//     let path = "levels/microban.json";
//     let content = std::fs::read_to_string(&path)?;
//     let level_file: LevelFile = serde_json::from_str(&content).expect("Failed to parse!");

//     let mut found = 0;
//     let mut missed = 0;

//     println!("Solving the {} map", level_file.name);
//     println!("---");
//     for (i, level) in level_file.levels.iter().enumerate() {
//         let level_str: Vec<&str> = level.lines.iter().map(|s| s.as_str()).collect();
//         let solution = solve(&level_str);
//         match solution {
//             Some(sol) => {
//                 found += 1;
//                 println!("level[{}]: {}", i, sol);
//             }
//             None => {
//                 missed += 1;
//                 eprintln!("level[{}]: no solution", i);
//             }
//         }
//     }

//     println!(
//         "Found: {}, Missed: {} -> {}%",
//         found,
//         missed,
//         (found / (found + missed)) * 100
//     );

//     Ok(())
// }
