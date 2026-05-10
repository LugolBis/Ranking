# Default configurations
RANKING_CONF ?= conf.json
RANKING_SIM ?= -s

SIM_PATH ?= Data/ncd/results.csv
SIM_EPSILON ?= 1e-12
SIM_SEED ?= 42

# Binaries
RANKING_BIN := ./target/release/ranking
SIM_BIN := ./target/release/simulation_plots

# Build
build-ranking:
	cargo build --release -p ranking

build-simulation:
	cargo build --release -p simulation_plots

build: build-ranking build-simulation

# Run
run-ranking:
	$(RANKING_BIN) -c $(RANKING_CONF) $(RANKING_SIM)

run-simulation:
	$(SIM_BIN) $(SIM_PATH) $(SIM_EPSILON) $(SIM_SEED)

# Full execution
all: build run-ranking run-simulation


.PHONY: \
	build-ranking \
	build-simulation \
	build \
	run-ranking \
	run-simulation \
	all