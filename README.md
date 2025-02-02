# Agave-Benchmarking-Solana-Core
This repository contains research, test findings, analysis, and results related to Solana Core, Solana validator performance and Agave-specific optimizations. It documents benchmarking experiments, statistical insights, and code changes made during testing.

# Goals of This Repository

- To systematically analyze performance optimizations for Agave and Solana.

- To document and compare benchmark results over different configurations.

- To provide statistical insights into throughput, latency, and resource utilization of transactions packets through the Solana-core.

- To maintain reproducibility of experiments.

# Setting Up the Environment

# Prerequisites

- Rust (latest stable)

- Criterion.rs (for benchmarking)

- Python (for data analysis & visualization).ou can also use Criterion for this.

# Installation

## Install Criterion for benchmarking
`cargo install cargo-criterion`

## Install Python dependencies (if needed)
`pip install matplotlib pandas numpy`


# Running Benchmarks

Benchmarks are executed using Criterion.rs.

To run a benchmark:

bash `cargo bench -p target folder --bench --test folder`

Results are stored in `./target/criterion` and can be viewed via HTML reports

# Data Collection & Analysis
Criterion provides visual tools to help interpret estimates

## Storing Benchmark Results

Benchmark results are saved in JSON format under benchmarks/results/. The repository includes scripts to process and visualize this data.

## Statistical Insights

Criterion automatically provides:

- Mean, median, variance, and standard deviation of throughput and latency.

- Confidence intervals for benchmark runs.

- Performance regression detection over time.

