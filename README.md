# Agave-Benchmarking-Solana-Core
This repository contains research, test findings, analysis, and results related to Solana Core, Solana validator performance and Agave-specific optimizations. It documents benchmarking experiments, statistical insights, and code changes made during testing.

# Repository Structure
/Agave-Benchmarking-Solana-Core
├── component-that-was-benchmarked/            # Criterion benchmarks and reports
    ├── agave-changes/         # Notes and diffs related to Agave modifications
    ├── analysis/              # Statistical analysis, graphs, and insights
    ├── reports/               # Markdown/PDF reports of findings
    ├── tests/                 # Custom test scripts and results
├── README.md              # Overview of the research and methodology
├── .gitignore             # Ignore unnecessary files

# Goals of This Repository

- To systematically analyze performance optimizations for Agave and Solana.

- To document and compare benchmark results over different configurations.

- To provide statistical insights into throughput, latency, and resource utilization.

- To maintain reproducibility of experiments.

# Setting Up the Environment

# Prerequisites

- Rust (latest stable)

- Criterion.rs (for benchmarking)

- Python (for data analysis & visualization)

# Installation

## Install Criterion for benchmarking
`cargo install cargo-criterion`

## Install Python dependencies (if needed)
`pip install matplotlib pandas numpy`


# Running Benchmarks

Benchmarks are executed using Criterion.rs.

To run a benchmark:

bash `cargo criterion`

Results are stored in `./target/criterion` and can be viewed via HTML reports

# Data Collection & Analysis

## Storing Benchmark Results

Benchmark results are saved in JSON format under benchmarks/results/. The repository includes scripts to process and visualize this data.

## Statistical Insights

Criterion automatically provides:

- Mean, median, variance, and standard deviation of throughput and latency.

- Confidence intervals for benchmark runs.

- Performance regression detection over time.

To extract key insights, use:
`python scripts/analyze_results.py`