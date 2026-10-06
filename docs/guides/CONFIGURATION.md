# Model & API Key Configuration Guide (MAGI System)

MAGI System implements a 100% vendor-agnostic HTTP transport based on the **OpenAI Chat Completions REST API** standard (`POST /chat/completions`). It requires no proprietary cloud SDKs and connects natively to any cloud provider, proxy gateway, or local inference engine (Ollama, LM Studio, vLLM).

MAGI supports two primary configuration modes in your `.env` file:
1. **Global Mode (Unified):** A single API Key, endpoint, and default model for the entire Trinity.
2. **Granular Mode (Per-Node):** Independent API keys, endpoints, and specialized models for each individual persona (**Melchior-1**, **Balthasar-2**, and **Casper-3**).

---

## Mode 1: Global Configuration (Unified)

Ideal if you have a single provider account or API key and want all three nodes to share the same connection or credit pool.

### Example 1A: OpenRouter (Single Global API Key)
In your repository root, edit or create `.env`:

```env
# Official OpenRouter base endpoint
MAGI_ENDPOINT=https://openrouter.ai/api/v1
MAGI_API_KEY=sk-or-v1-your-openrouter-key...

# Base model for all 3 nodes
MAGI_MODEL=anthropic/claude-3.5-sonnet
```
*(Note: If you specify `OPENROUTER_API_KEY=sk-or-...`, MAGI automatically detects and configures the endpoint to `https://openrouter.ai/api/v1` without requiring manual URL setup).*

### Example 1B: OpenRouter with Specialized Per-Node Models (Shared API Key)
You can maintain a single global OpenRouter API key while assigning optimized models to each persona:

```env
MAGI_ENDPOINT=https://openrouter.ai/api/v1
MAGI_API_KEY=sk-or-v1-your-global-key...

# Melchior-1 (The Scientist): Pure logic, architecture, and complexity
MELCHIOR_MODEL=cohere/north-mini-code:free

# Balthasar-2 (The Mother): Cybersecurity, threat models, and authoritative veto
BALTHASAR_MODEL=nvidia/nemotron-3-super-120b-a12b:free

# Casper-3 (The Woman): Pragmatism, DX, and delivery feasibility
CASPER_MODEL=openrouter/free
```

### Example 1C: 100% Offline Local Models (Zero API Key Needed)
If you run models locally using Ollama, LM Studio, or vLLM:

```env
# Ollama
MAGI_ENDPOINT=http://localhost:11434/v1
MAGI_MODEL=qwen2.5-coder:7b
MAGI_API_KEY=

# LM Studio
# MAGI_ENDPOINT=http://localhost:1234/v1
# MAGI_MODEL=local-model
```

---

## Mode 2: Granular Configuration (Per-Node)

Ideal if you maintain **separate API keys** (for tracking usage, enforcing distinct budget limits, or routing each node to different providers).

### Example 2A: OpenRouter with Granular API Keys
If you created 3 separate API keys in OpenRouter (one per node):

```env
MAGI_ENDPOINT=https://openrouter.ai/api/v1

# ==========================================
# MELCHIOR-1 (The Scientist)
# ==========================================
MELCHIOR_API_KEY=sk-or-v1-key-for-melchior...
MELCHIOR_MODEL=anthropic/claude-3.5-sonnet

# ==========================================
# BALTHASAR-2 (The Mother - Security Veto)
# ==========================================
BALTHASAR_API_KEY=sk-or-v1-key-for-balthasar...
BALTHASAR_MODEL=deepseek/deepseek-r1

# ==========================================
# CASPER-3 (The Woman - Pragmatism & DX)
# ==========================================
CASPER_API_KEY=sk-or-v1-key-for-casper...
CASPER_MODEL=openai/gpt-4o
```

> **Smart Auto-Resolution:** If any `{NODE}_API_KEY` variable begins with `sk-or-`, MAGI automatically routes that specific node to `https://openrouter.ai/api/v1`, even if no explicit endpoint was defined for that node.

### Example 2B: Multi-Provider Hybrid Configuration
You can freely mix local and cloud providers across personas:

```env
# Melchior runs locally on Ollama (Zero cost for architecture analysis)
MELCHIOR_ENDPOINT=http://localhost:11434/v1
MELCHIOR_MODEL=qwen2.5-coder:7b
MELCHIOR_API_KEY=

# Balthasar runs on OpenRouter with Claude (Maximum security rigor)
BALTHASAR_ENDPOINT=https://openrouter.ai/api/v1
BALTHASAR_MODEL=anthropic/claude-3.5-sonnet
BALTHASAR_API_KEY=sk-or-v1-balthasar-key...

# Casper runs directly on official DeepSeek
CASPER_ENDPOINT=https://api.deepseek.com/v1
CASPER_MODEL=deepseek-chat
CASPER_API_KEY=sk-deepseek-key...
```

---

## Environment Variables Reference Table

| Variable | Scope | Description | Default |
| :--- | :--- | :--- | :--- |
| `MAGI_ENDPOINT` | Global | OpenAI-compatible REST API base URL | `http://localhost:11434/v1` |
| `MAGI_MODEL` | Global | Model identifier for all 3 nodes | `default` |
| `MAGI_API_KEY` | Global | Authentication token (`Bearer <key>`) | `None` (Empty for local) |
| `MAGI_TIMEOUT_SECONDS` | Global | HTTP timeout in seconds per node evaluation | `60` (120 recommended for heavy tiers) |
| `OPENROUTER_API_KEY` | Fallback | If present, endpoint auto-resolves to OpenRouter | Auto-resolved |
| `MELCHIOR_API_KEY` | Granular | Dedicated API key for Melchior-1 | Inherits `MAGI_API_KEY` |
| `MELCHIOR_MODEL` | Granular | Dedicated model identifier for Melchior-1 | Inherits `MAGI_MODEL` |
| `MELCHIOR_ENDPOINT`| Granular | Dedicated base URL for Melchior-1 | Inherits `MAGI_ENDPOINT` |
| `BALTHASAR_API_KEY`| Granular | Dedicated API key for Balthasar-2 | Inherits `MAGI_API_KEY` |
| `BALTHASAR_MODEL` | Granular | Dedicated model identifier for Balthasar-2 | Inherits `MAGI_MODEL` |
| `BALTHASAR_ENDPOINT`| Granular| Dedicated base URL for Balthasar-2 | Inherits `MAGI_ENDPOINT` |
| `CASPER_API_KEY` | Granular | Dedicated API key for Casper-3 | Inherits `MAGI_API_KEY` |
| `CASPER_MODEL` | Granular | Dedicated model identifier for Casper-3 | Inherits `MAGI_MODEL` |
| `CASPER_ENDPOINT` | Granular | Dedicated base URL for Casper-3 | Inherits `MAGI_ENDPOINT` |
| `MAGI_LANG` | System | Output language for console & reports (`en`, `es`, or auto) | Auto |

---

## Verifying Your Configuration

To test your active configuration against an example proposal:

```powershell
# On Windows (PowerShell)
.\magi.ps1 idea test_idea.md

# On Linux / Docker
docker compose run --rm magi idea test_idea.md
```

You will see real-time NERV phosphor terminal monitors reflecting the exact model assigned to each node during both evaluation rounds.
