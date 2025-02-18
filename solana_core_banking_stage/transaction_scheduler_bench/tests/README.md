# High-Level Overview
This test benchmarks Solana's transaction scheduling mechanism in the banking stage by effectively simulating real-world scenarios and measuring critical performance metrics, which can help identify bottlenecks in the transaction pipeline. Emphasis is placed on benchmarking the performance of the banking stage and scheduler component, which is responsible for enqueueing transactions from one validator to others: The benchmark currently measures the time taken to push transactions into the `non_vote_sender` queue.

## Key Components
#### 1. BenchSetup Struct
The BenchSetup struct initializes the test environment. It includes:

- Exit Signal: Used to terminate services (exit).
- PohService: The Proof-of-History service for the validator.
- Forwarder: The component being benchmarked.
- UnprocessedTransactionStorage: Holds batches of unprocessed transactions.
- Metrics and Stats:
    - LeaderSlotMetricsTracker: Tracks metrics for leader slots.
    - BankingStageStats: Measures performance and execution statistics.
    - TracerPacketStats: Tracks packet statistics for tracing.
#### 2. setup Function
The setup function prepares the test environment:

- Genesis Configuration: Sets up the bank, ledger, and other Solana components.
- Transaction Creation: Creates packets containing transactions to simulate real-world scenarios.
- Channel Initialization: Creates different channels `unbounded`, `bounded(1024)` to simulate queues in maximum TPS conditions and also congestion and queueing delays 

## Detailed Walkthrough

The `b.iter` method executes the Criterion's bench function multiple times and gathers detailed metrics, such as mean, standard deviation, and iteration times.
Queues are created and transactions are forwarded to these queues to be sent to the banking stage
Latency Calculation:
Time for enqueueing the transactions is captured with `Instant::now()` and `elapsed()`.


#### Statistical Analysis:

Criterion automatically provides statistical insights, such as the mean and variance of throughput and latency, which are essential for this sort of tests.

`criterion_group!` and `criterion_main!` are used to define and run the benchmark tests.
The `forwarded` flag is reset after each iteration for reusability.
