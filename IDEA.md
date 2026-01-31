# Rumi-CLI: High-Context Local Agent Architecture

## 🎯 Core Vision
A cost-efficient, local-first development agent that uses small, high-performance LLMs (4B-8B parameters) to perform complex architectural and coding tasks. It leverages a semantic index (`MAP.md`) and "System 2" thinking workflows to maintain high accuracy within a local token economy.

## 🧠 Intelligence Engine
- **Local-First Inference:** Powered by **vLLM** (Port 8000) for high-throughput, OpenAI-compatible inference.
- **Small-Brain Iteration:** Optimized for 4B-8B models (e.g., Qwen3, Phi-4).
- **Dynamic Temperature Scaling:**
  - **Base Temp (0.7):** Standard for tool-calling and response generation.
  - **Max Temp (1.2):** Scaled automatically during long-running tool loops to prevent "stalling."
- **Structured JSON-Only Output:** The agent is strictly typed; raw text is treated as a fallback/error.

## 🛠️ Configuration & Customization (`rumi_config.json`)
The agent's behavior is fully externalized to avoid hardcoding:
- **LlmConfig:** API URLs, model IDs, temperature thresholds, and context window limits.
- **Prompts:** Customizable System Prompt, Intent Analysis rules, and Summary templates.
- **UI/Theming:** ANSI color codes for specific output types (e.g., "Dark Orange" for primary branding).

## 🗺️ Context Strategy (Lazy Loading)
To keep the context window slim and efficient for general tasks, Rumi uses **Lazy Context Loading**:
1. **Base Mode:** General questions (History, Trivia, Greetings) are answered using only the core System Prompt (~200 tokens).
2. **Context Injection:** When the Intent Analyzer detects a code-related query (`INFO_CODE` or `ACTION`), the `MAP.md` (~40k bytes) is injected into the system context on-the-fly.
3. **Token Economy:** This prevents "Context Fog" during general conversation while ensuring high-precision when coding.

## 📋 Intent Analysis (INFO vs ACTION)
Rumi-CLI moves beyond simple complexity checks to **Intent Classification**:
- **INFO_GENERAL:** Direct answers for non-technical queries.
- **INFO_CODE:** Analysis and explanation of the project structure using the codebase map.
- **ACTION:** Triggers the "Thinking Process" workflow for code generation, file management, or shell execution.

## 🧠 System 2: Thinking Engine (Action Planning)
For any `ACTION` intent, Rumi enters a separate environment to maintain structural integrity.
1. **Environment Setup:** Creates `thinking/TP_<ID>/input.md` with the full query and project map.
2. **Mandatory Planning:** The agent is redirected to read `input.md` and MUST write a `plan.md` before executing any modifications.
3. **Visual Confirmation:** The CLI provides real-time notifications when a plan is successfully finalized and saved to disk.

## 💾 Session & Context Lifecycle
- **Persistent Sessions:** Every interaction is logged to `sessions/SID_<timestamp>/history.json` for long-term recall and debugging.
- **Auto-Save:** History is flushed to disk after every turn.
- **Context Compression (`/compress`):** A specialized command that uses the LLM to summarize middle-history (retaining the System prompt and last 2 messages), preventing context window overflows.

## 🛠️ Toolbelt (Execution Loop)
The agent operates in a **Think -> Act -> Observe** loop using:
1. `read_file(path)`
2. `write_file(path, content)`
3. `run_shell(command)`
Results are fed back as "Observations" to allow for self-correction and iterative development.