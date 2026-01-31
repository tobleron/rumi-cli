# Rumi-CLI: High-Context Local Agent Idea

## 🎯 Core Vision
A cost-efficient, local-first development agent that uses small, high-performance LLMs (4B parameters) to perform complex architectural and coding tasks. It leverages a semantic index (`MAP.md`) to maintain a massive virtual context window while keeping the actual prompt context slim and efficient.

## 🧠 Intelligence Engine
- **Local-First:** Powered by **Ollama** for zero-cost, private, and offline inference.
- **Small-Brain Iteration:** Uses 4B models (e.g., Qwen2.5-Coder, Phi-3) as "thinking units" that iterate on logic. If a pattern is unknown, the model "re-invents" it through trial and error (compilation feedback loops).
- **Dynamic Temperature:**
  - **Temp 0.0:** For deterministic tasks like structured JSON tool calling and file parsing.
  - **Temp 0.7+:** For creative problem solving and architectural "thinking" phases.

## 🗺️ Context Management (MAP.md)
Instead of loading the entire codebase, Rumi-CLI uses a two-stage approach:
1. **Index Scan:** Read `MAP.md` to identify the "Feature Pods" and files relevant to the current request.
2. **Selective Loading:** Only pull the source code of the identified files into the context window.
3. **Token Economy:** Minimizes "Read Tax" and "Context Fog," allowing 4B models to stay focused.

## 🛠️ Execution Loop (Tool Calling)
The agent operates in a **Think -> Act -> Observe** loop:
1. **Structured Output:** LLM produces JSON-formatted tool calls.
2. **Rust Dispatcher:** The CLI parses JSON into Rust Enums, executes the system command (Read, Write, Shell, Compile), and captures the output.
3. **Feedback Injection:** Tool results are fed back into the next iteration's prompt, allowing the agent to self-correct.

## 🚀 Key Advantages
- **Cost Efficiency:** No API credits required.
- **Speed:** 4B models offer near-instantaneous inference on modern hardware.
- **Architectural Integrity:** By following `_dev-system` and `MAP.md` protocols, the agent adheres to project standards automatically.
