.PHONY: lint
lint:
	cargo clippy -- -W clippy::perf

.PHONY: build
build:
	cargo build --release

.PHONY: run
run: build
	@/bin/sh -c "time target/release/sokoban-solver"

.PHONY: bench
bench: build
	hyperfine ./target/release/sokoban-solver

.PHONY: profile
profile: build
	@echo "Running bytehound profiler..."
	MEMORY_PROFILER_LOG=warn LD_PRELOAD=/lib/libbytehound.so target/release/sokoban-solver

	@echo "Opening profiler output..."
	@bytehound server memory-profiling_sokoban-solver_*.dat
