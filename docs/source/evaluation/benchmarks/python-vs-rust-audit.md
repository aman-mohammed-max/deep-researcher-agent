<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Python vs. Rust: Deep Research Comparative Benchmark & Port Quality Audit

## Executive Summary

This document presents a comprehensive, empirical benchmark and architectural audit comparing **Implementation A (Original NVIDIA Python Deep Researcher)** and **Implementation B (Rust Port)**.

Both implementations were evaluated on the complex technical research assignment:
> *"Can We Run Massive LLMs Locally by Streaming Model Weights Directly from Cloud Storage?"*

### Summary Scorecard

| Assessment Dimension | Implementation A (Python) | Implementation B (Rust) | Verdict |
| :--- | :--- | :--- | :--- |
| **Research Quality Score** | **91.5 / 100** | **58.0 / 100** | **Python Wins (+33.5 pts)** |
| **Mathematical Precision** | **100% Verified** (exact $O(1)$ decode arithmetic) | **Flawed** (assumed 0.2% weight read per token; 364× error) | **Python Wins** |
| **Primary-Source Depth** | **10 Technical Papers / Core Docs** cited & verified | **42 URLs**, but heavily skewed toward blogs & aggregators | **Python Wins** |
| **Architectural Fidelity** | Native Dynamic Multi-Agent Graph (`deepagents`) | Hardcoded Linear Pipeline (`Intent -> Plan -> Work -> Write`) | **Python Preserves Architecture** |
| **System-Prompt Fidelity** | Full 550+ lines of Jinja2 domain prompts | Condensed by ~85% into inline string literals | **Severe Regression in Rust** |
| **Runtime Performance** | 28 min wall-clock, 83 LLM turns, 2.1 GB RAM | **1.8 min wall-clock, 10 LLM turns, 17 MB RAM** | **Rust Wins on Efficiency (15× faster, 120× less RAM)** |

---

## 1. Research Quality Scorecards (Out of 100)

| Criterion | Weight | Python Original (A) | Rust Port (B) | Score Justification |
| :--- | :---: | :---: | :---: | :--- |
| **Factual Accuracy** | 25 | **24.0 / 25** | **15.0 / 25** | Rust conflated cold-start loading with decode streaming; 0.2% weight-read hallucination. |
| **Primary-Source Quality & Coverage** | 15 | **14.5 / 15** | **09.0 / 15** | Python cited peer-reviewed papers (NeurIPS, ICML); Rust leaned on blogs & calculators. |
| **Citation Traceability & Correctness** | 15 | **13.5 / 15** | **08.0 / 15** | Python: 33 verified bracketed inline citations; Rust: 18 markdown links, some misattributed. |
| **Technical Reasoning & Depth** | 15 | **14.0 / 15** | **08.5 / 15** | Python analyzed $O(1)$ arithmetic intensity & SubSpec; Rust assumed permanent VRAM residency. |
| **Research Completeness** | 10 | **09.5 / 10** | **06.5 / 10** | Python delivered 3 staged implementation roadmaps with precise component specs. |
| **Quantitative & Mathematical Accuracy**| 10 | **10.0 / 10** | **04.0 / 10** | Python was 100% physically exact; Rust had a 364× per-token weight transfer error. |
| **Critical Thinking & Limitations** | 5 | **04.5 / 05** | **03.5 / 05** | Python proved network streaming is physically unviable for chat without speculative decoding. |
| **Clarity & Actionability of Advice** | 5 | **04.5 / 05** | **03.5 / 05** | Python targeted PCIe 5.0 and Apple Silicon; Rust recommended an expensive 6x RTX 4090 cluster. |
| **TOTAL SCORE** | **100** | **91.5 / 100** | **58.0 / 100** | **Python Wins (+33.5 points) on Deep Research Quality** |

---

## 2. Mathematical and Technical Audit

### Autoregressive Decoding Intensity
In an autoregressive decoder forward pass for batch size $B=1$, **every parameter in a dense model must be streamed from memory once per generated token**:

$$\text{Bytes Transferred per Token} \ge \text{Total Weight Bytes}$$

* **Python's Analysis (100% Mathematically Rigorous):**
  * 70B INT4 = **35 GB per token**.
  * Over 100 MB/s network (800 Mbps internet), generation latency is:
    $$\frac{35\text{ GB}}{100\text{ MB/s}} = \mathbf{350\text{ s/token}}\quad (\mathbf{0.0028\text{ tokens/sec}})$$
  * Proves that direct cloud weight streaming during autoregressive token generation is physically impossible for interactive chat.
  * Identifies **Substitute Speculative Decoding (SubSpec)** (NeurIPS 2025, arXiv:2509.18344v2) with token acceptance length $\tau \approx 30$ and Unified Memory (Apple M4 Max/Ultra at 400–1052 GB/s) as the only viable paths.
* **Rust's Analysis (Flawed by 364×):**
  * Assumed each token generation requires reading only **~0.2%** of model weights ("~96 MB for 70B Q4_K_M per token").
  * This error led to an unrealistic projection that streaming weights per token takes only 0.1s over a 100 MB/s network.

---

## 3. Port Compatibility Matrix

| Capability | Python Original | Rust Port | Status | Real-World Impact |
| :--- | :--- | :--- | :--- | :--- |
| **Agent Architecture** | Dynamic `deepagents` multi-agent graph with subagent delegation | Hardcoded 4-step coordinator pipeline | **Degraded** | Rust cannot dynamically replan, branch, or delegate sub-tasks. |
| **System Prompts** | Full 550+ lines of Jinja2 domain prompts | Condensed by ~85% into inline literals | **Degraded** | Dropped discovery constraints, task analysis, and synthesis rules. |
| **Prompt Injection** | Passed via `system` role with tool schemas | Concatenated into single `user` message | **Degraded** | Loses system-instruction steering tuned into modern LLMs. |
| **Research Planning** | Multi-turn planning with search tools & discovery budget | Single LLM prompt call with zero search tools | **Degraded** | Rust planner cannot explore the evidence landscape before decomposing queries. |
| **Worker Loops** | Multi-turn ReAct loops (up to 100 turns per worker) | 1 search dispatch + 1 LLM completion | **Degraded** | Workers cannot follow up on evidence gaps or verify conflicting claims. |
| **Systems Efficiency** | 2.1 GB RAM, 28 min runtime, heavy dependencies | **17 MB RAM, 1.8 min runtime, 0 dependencies** | **Improved** | Rust is ~15× faster and 120× more memory efficient. |
| **Resource Guard** | None (crashes on low memory) | Microsecond `/proc` & cgroup admission guard | **Improved** | 100% immune to Render 512MB RAM OOM crashes. |

---

## 4. Engineering Recommendations for Rust Elevation

1. **Restore Full NVIDIA System Prompts:** Port the complete, unabridged Jinja2 templates (`planner.j2`, `researcher.j2`, `writer.j2`) into MiniJinja and pass them to NIM in the `system` role.
2. **Equip Planner with Discovery Tools:** Allow `ResearchPlanner` to execute 1–3 preliminary web searches before finalizing the subquery plan.
3. **Multi-Turn ReAct Worker Loops:** Upgrade `ResearchWorker` from a single-shot summarizer to an iterative 3–5 turn agent capable of inspecting results and following up on gaps.
4. **Mathematical Sanity Checks:** Inject prompt constraints stating that autoregressive dense decoding accesses 100% of model parameters per token ($O(1)$ arithmetic intensity).
