use sokoban_solver::solve;

#[test]
fn test_microban() {
    let level = &[
        "####", "# .#", "#  ###", "#*@  #", "#  $ #", "#  ###", "####",
    ];
    let expected = "dlUrrrdLullddrUluRuulDrddrruLdlUU";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_petitesse() {
    let level = &["#####", "#   #", "#.$.#", "# $ #", "#+$ #", "#####"];
    let expected = "uuurrdddLruuullddRluurrdLddrU";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_scoria() {
    let level = &[
        "  ####", "  #  #", "### .#", "#  * #", "# #@ #", "# $* #", "##   #", " #####",
    ];
    let expected = "UdrddlUruulllddRdrruuuuulDrddddlluRdrUUULDrddlluluuR";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_autogen() {
    let level = &[
        "########", "###  . #", "## * # #", "## .$  #", "##  #$##", "### @ ##", "########",
        "########",
    ];
    let expected =
        "luuluRurrrddlLrruullldlddrdrrUdlluluururrrddLddlluUddrruuLUdrddlluluuRuRDllddrUddrruuL";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_squared() {
    let level = &[
        "#######", "# . * #", "#.*$ .#", "# $ $ #", "#*$ .*#", "#@* * #", "#######",
    ];
    let expected = "UURURRDLdLU";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_boring2() {
    let level = &[
        "#######", "#  .+.#", "#.*.####", "# $ $..#", "# $#$$ #", "#*$ $  #", "#      #",
        "########",
    ];
    let expected = "lddRRDrddlUdlluRdlllUUURlddRluurDuRuullDRdrUluRddldlddrUUddrruLdlluurUruuRllldRdrUluRddldlddrUUddrrrruLLLdlluurUruulldRddlddrUUUrRllUluRddrrDulldddrrruLLrrruLddlllluururrDrdLdllluuUdddrUUddrruLdlluurUrrddlL";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}
