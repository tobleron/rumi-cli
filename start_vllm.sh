#!/bin/bash
set -x

# Configuration for vLLM with Qwen3 4B GPTQ
# Using 60% GPU memory to leave room for desktop usage.

# Ensure we are in the project root
cd "$(dirname "$0")"

# Activate vLLM virtual environment
source ~/venvs/vllm/bin/activate

echo "Starting vLLM Server..."
echo "Model Path: ./LLM/qwen3-4b-gptq"
echo "Port: 8000"

python3 -u -m vllm.entrypoints.openai.api_server \
  --model ./LLM/qwen3-4b-gptq \
  --quantization gptq \
  --dtype float16 \
  --gpu-memory-utilization 0.8 \
  --max-model-len 24576 \
  --max-num-seqs 4 \
  --enforce-eager \
  --disable-custom-all-reduce \
  --port 8000 \
  --served-model-name qwen3-4b
