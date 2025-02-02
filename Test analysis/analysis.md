# Analysis on Test Results

This repo contains  test code analysis for evaluating the correlation between throughput and latency. The analysis is performed using Criterion, leveraging statistical methods to determine the relationship between these two metrics. The primary goal is to assess how changes in throughput affect latency within the banking stage, transaction scheduler and snapshot packaging services.

## Forwarder
Key insights from data (You can find a dataset sample in `banking-stage-forwarder-test/reports_banking_stage_forwarder`)

- Average Throughput: ~501,937 TPS
- Average Latency: ~2.03 ms
- Minimum Throughput: 293,395.99 TPS
- Maximum Throughput: 616,040.16 TPS
- Minimum Latency: 1.75 ms
- Maximum Latency: 3.41 ms

The throughput fluctuated significantly, with occasional drops below 300K TPS and peaks exceeding 600K TPS. Latency remains relatively stable, mostly hovering around 2 ms, with some spikes beyond 3 ms.

### Statistical observations

**Mean Execution Time:**
- Estimated Value: ~20.93 million (20,934,561.68)
- Confidence Interval (95%): Between 20.60M and 21.31M
- Standard Error: 181,000
**This suggests that, on average, the execution time is fairly stable with a small margin of error.**

**Median Execution Time:**
- Estimated Value: ~20.62 million (20,618,371.83)
- Confidence Interval (95%): Between 20.23M and 20.94M
- Standard Error: 160,535
**The median is slightly lower than the mean, which likely indicates a right-skewed distribution (some high values could be pulling the mean up).**

**Median Absolute Deviation (MAD):**
- Point Estimate: ~1.39M
- Confidence Interval (95%): Between 1.04M and 1.62M
- Standard Error: 149,893
**This suggest there is a high degree of consistency between the varying outputs (Throughput and latency) but the discrepncy between the MAD and SD suggests some fluctuations**

**Standard Deviation (Overall Variability):**
- Estimated Value: ~1.82M
- Confidence Interval (95%): Between 1.27M and 2.32M
- *Standard Error: 270,317
**The relatively high standard deviation suggests that execution time varies significantly across runs.**

The upper bound of the execution time (21.31M) is significantly higher than the lower bound (20.60M), implying that some runs take longer than others.
Possible causes could include CPU contention, differences in transaction scheduling, Background OS processes impacting execution

#### Questions

-Will adjusting restrictive data budget impact these results significantly?
-Data packets are set to `10240` packets per batch. Is this the average number of packets the forwarder is expected to process under real conditions?