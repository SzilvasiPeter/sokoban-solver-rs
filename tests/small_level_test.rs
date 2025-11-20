use sokoban_solver::solve;

#[test]
fn test_microban() {
    let level = &[
        "####", "# .#", "#  ###", "#*@  #", "#  $ #", "#  ###", "####",
    ];
    let expected = "ULURDLUU";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_petitesse() {
    let level = &["#####", "#   #", "#.$.#", "# $ #", "#+$ #", "#####"];
    let expected = "LRLU";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_scoria() {
    let level = &[
        "  ####", "  #  #", "### .#", "#  * #", "# #@ #", "# $* #", "##   #", " #####",
    ];
    let expected = "UURDRUUULDR";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_autogen() {
    let level = &[
        "########", "###  . #", "## * # #", "## .$  #", "##  #$##", "### @ ##", "########",
        "########",
    ];
    let expected = "RLULULURRDUL";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}

#[test]
fn test_squared() {
    let level = &[
        "#######", "# . * #", "#.*$ .#", "# $ $ #", "#*$ .*#", "#@* * #", "#######",
    ];
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
    let expected = "RRDURUUURRDRDRURUULURRURUULLLURUUURURDLLLDLUUULUL";

    let actual = solve(level).expect("No solution found");
    assert_eq!(actual, expected);
}
