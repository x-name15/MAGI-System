# Model & Environment Configuration Guide (MAGI System)

MAGI System implements a 100% vendor-agnostic HTTP transport based on the **OpenAI Chat Completions REST API** standard (`POST /chat/completions`). It requires no proprietary cloud SDKs and connects natively to any cloud provider, proxy gateway, or local inference engine (Ollama, LM Studio, vLLM).

MAGI supports two primary configuration paradigms in your `.env` file:
1. **Global Mode (Unified):** A single API Key, endpoint, fallback model, and subagent worker for the entire Trinity.
2. **Granular Mode (Per-Node):** Independent API keys, endpoints, primary models, hot-standby fallback models, and subagents for each individual persona (**Melchior-1**, **Balthasar-2**, and **Casper-3**).

---

## 1. Complete Environment Reference

| Environment Variable | Description | Default | Node Overrides |
| :--- | :--- | :--- | :--- |
| `MAGI_ENDPOINT` | Base URL for LLM Chat Completions endpoint | `https://openrouter.ai/api/v1` | `MELCHIOR_ENDPOINT`, `BALTHASAR_ENDPOINT`, `CASPER_ENDPOINT` |
| `MAGI_API_KEY` | Bearer API token for authentication | None | `MELCHIOR_API_KEY`, `BALTHASAR_API_KEY`, `CASPER_API_KEY` |
| `MAGI_MODEL` | Primary model identifier | `openai/gpt-4o-mini` | `MELCHIOR_MODEL`, `BALTHASAR_MODEL`, `CASPER_MODEL` |
| `MAGI_FALLBACK_MODEL` | Hot-standby failover model used when primary fails | None | `MELCHIOR_FALLBACK_MODEL`, `BALTHASAR_FALLBACK_MODEL`, `CASPER_FALLBACK_MODEL` |
| `MAGI_SUBAGENT_MODEL` | Worker model for OpenRouter server tools delegation | None | `MELCHIOR_SUBAGENT_MODEL`, `BALTHASAR_SUBAGENT_MODEL`, `CASPER_SUBAGENT_MODEL` |
| `MAGI_SUBAGENT_WEB_SEARCH` | Enables web search tool for subagent workers | `false` | `MELCHIOR_SUBAGENT_WEB_SEARCH`, etc. |
| `MAGI_WEB_SEARCH` | Enables web search tool for primary node evaluations | `false` | `MELCHIOR_WEB_SEARCH`, etc. |
| `MAGI_MAX_TOKENS` | Maximum completion token budget per generation | `4096` | `MELCHIOR_MAX_TOKENS`, `BALTHASAR_MAX_TOKENS`, `CASPER_MAX_TOKENS` |
| `MAGI_MAX_CONTEXT_CHARS`| Character budget for payload before smart 60/40 truncation| `60000` | N/A |
| `MAGI_MAX_RETRIES` | Max retries for transient HTTP errors (429, 5xx) | `3` | N/A |
| `MAGI_RETRY_DELAY_MS` | Initial backoff delay in milliseconds | `1000` | N/A |
| `MAGI_ALLOW_DEGRADED_QUORUM`| Enables 2-of-3 majority consensus if 1 node drops offline | `true` | N/A |
| `MAGI_TIMEOUT_SECONDS` | Maximum timeout in seconds for complete deliberation | `360` | N/A |
| `MAGI_LANG` | Enforces display & report language (`es` or `en`) | Auto-detect | N/A |
| `MAGI_AUTHOR` | Author identity string recorded in SpacetimeDB audit logs | `developer@magi` | N/A |
| `SPACETIMEDB_URI` | URL of the local SpacetimeDB instance | `http://spacetimedb:3000` (Docker) | N/A |
| `SPACETIMEDB_DATABASE`| Target database module name | `magi-system` | N/A |

> **Smart Auto-Resolution:** If `MAGI_API_KEY` or any `{NODE}_API_KEY` begins with `sk-or-`, MAGI automatically sets the endpoint to `https://openrouter.ai/api/v1` without requiring manual URL configuration.

---

## 2. Configuration Examples

### Example 1: OpenRouter with Granular Paid Personas & Free Subagent Worker
This is the recommended production pair-programming setup: high-quality specialized primary models backed by a fast, free worker model for delegated sub-tasks:

```env
SPACETIMEDB_URI=http://spacetimedb:3000
SPACETIMEDB_DATABASE=magi-system
MAGI_LANG=es
MAGI_MAX_TOKENS=4096

# Subagent Worker Model (Used by all nodes for micro-level analytical delegation)
MAGI_SUBAGENT_MODEL=cohere/north-mini-code:free

# ==============================================================================
# MELCHIOR-1 (The Scientist: Logic, Architecture & Algorithms)
# ==============================================================================
MELCHIOR_API_KEY=sk-or-v1-melchior-key...
MELCHIOR_MODEL=deepseek/deepseek-v4.1-flash
MELCHIOR_FALLBACK_MODEL=cohere/north-mini-code:free

# ==============================================================================
# BALTHASAR-2 (The Mother: Cybersecurity & Authoritative Veto)
# ==============================================================================
BALTHASAR_API_KEY=sk-or-v1-balthasar-key...
BALTHASAR_MODEL=openai/gpt-5.1-codex-mini
BALTHASAR_FALLBACK_MODEL=nvidia/nemotron-3-super-120b-a12b:free

# ==============================================================================
# CASPER-3 (The Woman: Pragmatism, DX & Delivery)
# ==============================================================================
CASPER_API_KEY=sk-or-v1-casper-key...
CASPER_MODEL=google/gemini-3.1-flash-lite
CASPER_FALLBACK_MODEL=dots-studio/dots-3-note-preview:free
```

---

### Example 2: 100% Free Cloud Setup (OpenRouter Free Tier)
Deploy MAGI with zero credit card usage relying on free OpenRouter endpoints:

```env
MAGI_ENDPOINT=https://openrouter.ai/api/v1
MAGI_API_KEY=sk-or-v1-your-free-key...
MAGI_FALLBACK_MODEL=cohere/north-mini-code:free

MELCHIOR_MODEL=cohere/north-mini-code:free
BALTHASAR_MODEL=nvidia/nemotron-3-super-120b-a12b:free
CASPER_MODEL=dots-studio/dots-3-note-preview:free
```

---

### Example 3: 100% Offline Local Inference (Zero API Keys Needed)
Run entirely air-gapped on your workstation using local Ollama models:

```env
# Point all nodes to local Ollama daemon
MAGI_ENDPOINT=http://localhost:11434/v1
MAGI_API_KEY=
MAGI_MODEL=qwen2.5-coder:7b

# Or assign distinct local models per persona:
MELCHIOR_MODEL=deepseek-r1:8b
BALTHASAR_MODEL=llama3.1:8b
CASPER_MODEL=mistral:7b
```
*(Also compatible with LM Studio on `http://localhost:1234/v1`, vLLM on `http://localhost:8000/v1`, or LocalAI).*

---

### Example 4: Hybrid Architecture (Local Developer + Cloud Security)
Mix local models for rapid architecture audits while routing security audits to a frontier cloud model:

```env
# Melchior runs locally on Ollama (Zero API cost for architecture analysis)
MELCHIOR_ENDPOINT=http://localhost:11434/v1
MELCHIOR_MODEL=qwen2.5-coder:7b
MELCHIOR_API_KEY=

# Balthasar runs on Anthropic Claude 3.5 Sonnet for deep zero-trust security audits
BALTHASAR_ENDPOINT=https://openrouter.ai/api/v1
BALTHASAR_API_KEY=sk-or-v1-balthasar-key...
BALTHASAR_MODEL=anthropic/claude-3.5-sonnet

# Casper runs locally on Ollama for pragmatic DX review
CASPER_ENDPOINT=http://localhost:11434/v1
CASPER_MODEL=mistral:7b
CASPER_API_KEY=
```

---

## 3. Hot-Standby Failover Behavior

When `MAGI_FALLBACK_MODEL` (or `{NODE}_FALLBACK_MODEL`) is configured:
1. The dispatcher attempts generation with the primary model.
2. If the primary model fails with HTTP 402 (Payment Required / Out of Credits), persistent 429 rate limits, connection resets, or 5xx server errors across all retries, the failover circuit engages.
3. The node logs: `[{node_id}] Primary model ({model}) failed: {err}. Engaging backup circuit ({backup})...`.
4. The fallback model is dispatched with identical prompts and schemas.
5. In SpacetimeDB history and Markdown reports, the model is permanently marked as `<model> (backup)`.
