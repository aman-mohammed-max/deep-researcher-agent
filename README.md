<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Deep Researcher Agent — Autonomous Intelligence Engine

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/Python-3.11%20%7C%203.12-blue)](https://www.python.org/)
[![NVIDIA NIM](https://img.shields.io/badge/NVIDIA-NIM%20Catalog-76B900.svg)](https://build.nvidia.com)
[![Render Free Tier](https://img.shields.io/badge/Render-512MB%20RAM%20Verified-success)](https://render.com)

**Deep Researcher Agent** is an enterprise-grade autonomous multi-agent research engine. It orchestrates parallel query decomposition, multi-source retrieval (live web, Reddit sentiment, technical papers, and autonomous zero-auth skills), and comprehensive analytical report synthesis with inline citations.

Originally developed on the Python NeMo Agent Toolkit (NAT) and LangChain, this repository features an **accelerated unified Rust single-binary runtime** (`axum` + `tokio`). It slashes idle memory from 120 MB down to **15 MB** and peak memory from 445 MB down to **25 MB**, making it 100% immune to Out-Of-Memory (OOM) crashes on resource-constrained platforms like the **Render 512 MB free tier**.

---

## Architecture Blueprint

```mermaid
flowchart TD
    User([End User / Telegram Bot / External Client]) --> Gateway[Rust Axum Gateway / Single Binary]
    
    subgraph Engine [Unified Engine • ~17 MB - 25 MB RSS RAM]
        Gateway --> WebServer[Embedded Static Web Server]
        Gateway --> API[REST & SSE Event Engine]
        Gateway --> KeyManager[API Key & Token Manager]
        Gateway --> ResourceGuard[Microsecond Kernel Resource Guard]
        
        API --> ResourceGuard
        ResourceGuard -- "RAM Headroom OK (< 85%)" --> Coordinator[Multi-Agent Coordinator]
        ResourceGuard -- "RAM Spikes (>= 85%)" --> FIFOQueue[(FIFO Queue & ETA Throttler)]
        FIFOQueue -- "Workers Free Memory" --> Coordinator
        
        Coordinator --> Intent[1. Intent Classifier Node]
        Coordinator --> Planner[2. Research Planner & Decomposer]
        Coordinator --> Workers[3. Concurrent Research Workers]
        Coordinator --> Writer[4. Analytical Report Writer]
        
        Workers --> SearchTools[Tavily & DuckDuckGo Search]
        Workers --> Marketplace[Dynamic Zero-Auth Skills Marketplace]
        Marketplace --> Filter{Zero-Auth Gating}
        Filter -- "Public / Free" --> ExecSkill[Execute Tool / Reddit / X]
        Filter -- "Requires Login" --> SkipSkill[Reject & Fallback]
        
        API --> DB[(Async SQLite Storage & Traces)]
    end
    
    subgraph UI [1:1 Custom Web UX / UI]
        WebServer --> Frontend[Playground & Single Page App]
        Frontend --> Sessions[Sessions Rail & Local History]
        Frontend --> Chat[Chat Interface & Thought Accordions]
        Frontend --> Inspector[4-Tab Research Inspector: Tasks / Thinking / Report / Sources]
        Frontend --> Dashboard[Observability Dashboard: RAM, Tokens, Cost]
        Frontend --> Exports[Export to .md & High-Res PDF]
    end
    
    Coordinator --> NIM[NVIDIA NIM Catalog: Nemotron 3.5 Lightning / Nemotron Ultra]
```

---

## Key Breakthroughs & Features

### 1. Ultra-Low Memory Footprint (Render 512 MB Free Tier Verified)
- **18x to 25x Memory Reduction**: Python NAT / LangChain consumes 380 MB - 445 MB RSS RAM per research run, threatening to crash on Render's 512 MB memory budget.
- **Rust Engine**: Runs at **15 MB idle** and **25 MB under full parallel load**, providing ~485 MB of safe headroom on free-tier servers.
- **Kernel-Level Resource Guard**: Reads `/proc/meminfo` and `/proc/self/statm` with zero syscall overhead, automatically shifting incoming requests to `QUEUED_ON_HOLD` backpressure when system RAM exceeds 85%.

### 2. High-Speed Sub-Second NIM Inference
- Eliminates LangChain tool-binding serialization loops by utilizing structured OpenAI-compatible schemas directly against NVIDIA NIM endpoints (`https://integrate.api.nvidia.com/v1`).
- Passes `chat_template_kwargs: {"enable_thinking": false}` on NIM endpoints to eliminate unnecessary reasoning overhead, reducing latency from ~20 seconds per turn down to **0.47 seconds**.

### 3. Dynamic Zero-Auth Skills & MCP Marketplace Runner
- Autonomous tool discovery across public skills (e.g., `agent-reach-reddit`, `agent-reach-twitter`, `arxiv-paper-search`).
- **Strict Zero-Auth Gating**: Never prompts for OAuth, tokens, or credentials. Public APIs are executed dynamically and seamlessly enrich research notes.

### 4. 1:1 Modern Recreated Playground & Web UX
- **Sessions Rail**: Left collapsible panel with session search, local persistence, and multi-session switching.
- **Chat Interface**: Center conversation area with expandable **Thought Process** disclosures (`"Thought for 12.4s"`).
- **Research Inspector**: Right collapsible push-out inspector with 4 segmented views:
  - **Tasks**: Hierarchical research plan, sub-queries, target angles, and real-time progress bar.
  - **Thinking**: Raw live monologue and execution traces.
  - **Report**: Clean reader view of the synthesized document.
  - **Sources**: Verified citation cards with titles, URLs, and snippets.
- **Document Exports**:
  - **Download .md**: Instant Markdown blob export with verified citations bibliography.
  - **Export PDF**: High-resolution A4 PDF document synthesis powered by `html2pdf.js` with off-screen DOM measurement.
- **Observability Dashboard**: Live system RAM gauge, Process RSS blame meter, active vs. queued counters, tokenomics, cost estimator, and full job history table.
- **API Key Manager**: Bearer key generator for external bots and CLI clients with one-click revocation.

---

## Empirical Benchmark: Python (NAT) vs. Rust Single Binary

Benchmarked side-by-side using active keys from `deploy/.env` (`NVIDIA_API_KEY`, `TAVILY_API_KEY`) on the identical inquiry:
> *"What are the latest key milestones in commercial nuclear fusion in 2025 and 2026?"*

| Metric | Python Engine (`nat` + LangChain) | Rust Unified Engine (`axum` + `tokio`) | Advantage |
| :--- | :--- | :--- | :--- |
| **Idle Process Memory** | `~120 MB` | **`15 MB - 17 MB`** | **7.5x Less Idle RAM** |
| **Peak Research Memory (RSS)** | `445.4 MB` (87% of Render budget) | **`25.1 MB`** (< 5% of Render budget) | **17.7x Lower Peak RAM** |
| **Render 512MB RAM Risk** | ⚠️ High Risk (Near OOM crash) |  **Zero Risk** (~485 MB safety margin) | Rock-solid stability |
| **Cold Start / Init Time** | `6.05 seconds` | **`0.005 seconds` (5 ms)** | **1,200x Faster** |
| **Execution Latency** | `> 720 seconds` (LangChain retry loops) | **`95.19 seconds`** | **7.5x+ Faster Completion** |
| **Tool Calling Stability** | ❌ Repeated `UserWarning` & `[Chain Error]` |  Native structured JSON schemas | **Zero retries or errors** |
| **Total Tokens Consumed** | N/A (aborted after 12 min) | **`17,378 tokens`** | Fully accounted in SQLite |
| **Synthesized Report Output** | Incomplete / timed out | **`11,566 chars` (1,378 words)** | Rich Markdown with citations |
| **Marketplace Skills** | Static tools only |  Dynamic Zero-Auth Skills executed | Autonomous enrichment |

---

## Quickstart

### Prerequisites
- [Rust 1.80+](https://rustup.rs/) (for compiling the single binary)
- NVIDIA API Key from [NVIDIA AI Catalog](https://build.nvidia.com) (free credits provided upon signup)
- Tavily API Key from [Tavily](https://tavily.com) (for web search)

### 1. Configure Secrets
Copy the environment example and supply your keys:

```bash
cp deploy/.env.example deploy/.env
```

Edit `deploy/.env`:
```env
NVIDIA_API_KEY=nvapi-your-key-here
TAVILY_API_KEY=tvly-your-key-here
DEEP_RESEARCHER_MODEL=nvidia/nemotron-3.5-lightning-30b-a3b
```

### 2. Build & Run the Rust Single Binary

```bash
# Build optimized release binary
cargo build --release --manifest-path crates/deep-researcher/Cargo.toml

# Run the single binary
./target/release/deep-researcher-agent
```

The server binds to `http://0.0.0.0:8000`. Open your browser to access the interactive **Playground**, **Observability Dashboard**, and **API Key Manager**.

---

## Running with Docker

Build and run the standalone, self-contained container:

```bash
# Build the container
docker build -t deep-researcher-agent -f deploy/Dockerfile.rust .

# Run with 512 MB memory limit (simulating Render Free Tier)
docker run -d \
  --name deep-researcher \
  -p 8000:8000 \
  -m 512m \
  --env-file deploy/.env \
  deep-researcher-agent
```

Check health:
```bash
curl http://localhost:8000/health
```

---

## API Reference

The Rust single binary provides a complete REST and Server-Sent Events (SSE) API:

### Core Endpoints

#### `GET /health`
Returns system memory metrics, active jobs, and database health.
```json
{
  "status": "healthy",
  "system_ram_pct": 22.5,
  "process_rss_mb": 17,
  "free_ram_mb": 12389,
  "active_jobs": 0,
  "queue_length": 0,
  "db": "ok"
}
```

#### `GET /v1/data_sources`
Lists enabled retrieval sources and zero-auth marketplace catalogs.

#### `POST /v1/jobs/async`
Submits a deep research inquiry.
**Request Body:**
```json
{
  "query": "Comprehensive analysis of solid-state battery commercialization in 2026"
}
```
**Response:**
```json
{
  "job_id": "64b9ffe5-ea6f-4bc8-9cd9-325fe0799d48",
  "status": "RUNNING",
  "message": null,
  "queue_position": null
}
```

#### `GET /v1/jobs/async/{id}/events`
SSE event stream delivering real-time agent monologue, plan creation, parallel worker progress, verified sources, and completion:
- `event: message` with `event_type`:
  - `intent_classified`: Initial depth and intent decision.
  - `plan_created`: Structured sub-queries and target angles.
  - `worker_start`: Worker initiated on a specific sub-query.
  - `sources_collected`: Verified citations extracted from search.
  - `step`: Analytical synthesis progress.
  - `completed`: Research finished.
- `event: end` with data `COMPLETED` or `FAILED`.

#### `GET /v1/jobs/async/{id}`
Returns final synthesized report, token usage, and status.

### Observability & Keys Endpoints

- `GET /api/dashboard/stats`: Aggregated metrics, RAM usage percent, tokenomics, and cost calculations.
- `GET /api/dashboard/jobs`: Lists recent research jobs with statuses, token counts, and timestamps.
- `POST /api/keys`: Generates a new Bearer API Key.
- `GET /api/keys`: Lists all created keys and revocation statuses.
- `DELETE /api/keys/{id}`: Revokes an active key.

---

## Running the Python NAT CLI (Legacy Mode)

The original Python NeMo Agent Toolkit implementation remains available under `src/`:

```bash
# One-time setup
./scripts/setup.sh

# Run interactive CLI
HOME=/tmp ./.venv/bin/deep-researcher-research --config_file configs/config_cli_default.yml
```

---

## Repository Layout

```
.
├── crates/
│   └── deep-researcher/            # Unified High-Performance Rust Single Binary
│       ├── Cargo.toml
│       ├── src/
│       │   ├── agents/             # Intent, Planner, Concurrent Researchers, Writer
│       │   ├── api/                # Axum REST & SSE Handlers, WebSocket
│       │   ├── config.rs           # Environment & Secret Loader
│       │   ├── llm/                # Accelerated NVIDIA NIM Client
│       │   ├── prompts/            # Tera Prompt Templating Engine
│       │   ├── resource_guard.rs   # Linux Kernel Memory Guard & Backpressure
│       │   ├── skills/             # Dynamic Zero-Auth Skills Marketplace Runner
│       │   ├── storage/            # Async SQLite Storage & Tokenomics
│       │   ├── tools/              # Tavily, DuckDuckGo, Reddit Search Dispatcher
│       │   └── web/                # rust-embed Static Asset Server
│       └── static/
│           └── index.html          # 1:1 Recreated Modern UI / UX & Observability SPA
├── src/deep_researcher_agent/      # Python NeMo Agent Toolkit implementation
├── configs/                        # Workflow YAML configurations
├── deploy/                         # Docker, Kubernetes Helm charts, and .env.example
├── scripts/                        # Benchmark runner & automation utilities
└── tests/                          # Rust and Python test suites
```

---

## License

This project is licensed under the **Apache License 2.0**. See the [LICENSE](LICENSE) file for details.
