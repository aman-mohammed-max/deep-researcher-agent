<!--
SPDX-FileCopyrightText: Copyright (c) 2025-2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Benchmarks

Standardized evaluation suites for measuring research quality.

| Benchmark | What it measures | Dataset size | Agents tested |
|-----------|-----------------|--------------|---------------|
| [FreshQA](./freshqa.md) | Factual accuracy on current knowledge | 600 questions | Shallow, Full pipeline |
| [Deep Research Bench](./deep-research-bench.md) | Report quality (RACE + FACT metrics) | 100 topics | Deep researcher |
| [DeepSearchQA](./deepsearch-qa.md) | Document QA across categories | 900 problems | Deep researcher |
| [Python vs. Rust Port Audit](./python-vs-rust-audit.md) | Architectural fidelity, math precision, and research quality | 2 comprehensive runs | Python & Rust engines |

```{toctree}
:titlesonly:

freshqa.md
deep-research-bench.md
deepsearch-qa.md
python-vs-rust-audit.md
```
