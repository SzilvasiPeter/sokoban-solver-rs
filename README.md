# sokoban-solver-rs
Sokoban solver written in Rust

# lint

```sh
cargo clippy -- -W clippy::perf
```

# build

```sh
cargo build --release
```

# benchmark

```sh
hyperfine target/release/sokoban-solver
```

# profiling

Install with `yay -Sy bytehound-bin` command.

```sh
export MEMORY_PROFILER_LOG=warn
LD_PRELOAD=/lib/libbytehound.so target/release/sokoban-solver
bytehound server memory-profiling_sokoban-solver_*.dat
```

Go to <http://localhost:8080/>
