#!/bin/bash
# Download Moonshine tiny model for hammertalk

MODEL_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hammertalk/models/moonshine-tiny"

mkdir -p "$MODEL_DIR"
cd "$MODEL_DIR" || exit 1

BASE_URL="https://huggingface.co/UsefulSensors/moonshine/resolve/main/onnx/merged/tiny"

echo "Downloading Moonshine tiny model to $MODEL_DIR..."

# ONNX models are in the float subdirectory
for file in encoder_model.onnx decoder_model_merged.onnx; do
    if [[ -f "$file" ]]; then
        echo "  $file already exists, skipping"
    else
        echo "  Downloading $file..."
        curl -fL "$BASE_URL/float/$file" -o "$file" || {
            echo "Failed to download $file" >&2
            exit 1
        }
    fi
done

# Tokenizer is in the separate moonshine-tiny repo
if [[ -f "tokenizer.json" ]]; then
    echo "  tokenizer.json already exists, skipping"
else
    echo "  Downloading tokenizer.json..."
    curl -fL "https://huggingface.co/UsefulSensors/moonshine-tiny/resolve/main/tokenizer.json" -o "tokenizer.json" || {
        echo "Failed to download tokenizer.json" >&2
        exit 1
    }
fi

echo "Done! Model files:"
ls -lh "$MODEL_DIR"
