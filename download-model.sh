#!/bin/bash
# Download transcription models for hammertalk
#
# Usage: ./download-model.sh [ENGINE]
#   Engines: moonshine-tiny, moonshine-base,
#            whisper-tiny, whisper-base, whisper-small, whisper-medium,
#            whisper-large-v3, whisper-large-v3-turbo, all
# Default: moonshine-tiny

set -e

MODELS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hammertalk/models"
WHISPER_BASE_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

download_moonshine_tiny() {
    local model_dir="$MODELS_DIR/moonshine-tiny"
    mkdir -p "$model_dir"
    cd "$model_dir"

    local base_url="https://huggingface.co/UsefulSensors/moonshine/resolve/main/onnx/merged/tiny"

    echo "Downloading Moonshine tiny model to $model_dir..."

    for file in encoder_model.onnx decoder_model_merged.onnx; do
        if [[ -f "$file" ]]; then
            echo "  $file already exists, skipping"
        else
            echo "  Downloading $file..."
            curl -fL "$base_url/float/$file" -o "$file" || {
                echo "Failed to download $file" >&2
                exit 1
            }
        fi
    done

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
    ls -lh "$model_dir"
}

download_moonshine_base() {
    local model_dir="$MODELS_DIR/moonshine-base"
    mkdir -p "$model_dir"
    cd "$model_dir"

    local base_url="https://huggingface.co/UsefulSensors/moonshine/resolve/main/onnx/merged/base"

    echo "Downloading Moonshine base model to $model_dir..."

    for file in encoder_model.onnx decoder_model_merged.onnx; do
        if [[ -f "$file" ]]; then
            echo "  $file already exists, skipping"
        else
            echo "  Downloading $file..."
            curl -fL "$base_url/float/$file" -o "$file" || {
                echo "Failed to download $file" >&2
                exit 1
            }
        fi
    done

    if [[ -f "tokenizer.json" ]]; then
        echo "  tokenizer.json already exists, skipping"
    else
        echo "  Downloading tokenizer.json..."
        curl -fL "https://huggingface.co/UsefulSensors/moonshine-base/resolve/main/tokenizer.json" -o "tokenizer.json" || {
            echo "Failed to download tokenizer.json" >&2
            exit 1
        }
    fi

    echo "Done! Model files:"
    ls -lh "$model_dir"
}

download_whisper_model() {
    local variant="$1"    # e.g. tiny, base, small, medium
    local filename="$2"   # e.g. ggml-tiny.en.bin, ggml-large-v3.bin
    local filepath="$MODELS_DIR/$filename"

    mkdir -p "$MODELS_DIR"

    echo "Downloading Whisper $variant model to $filepath..."

    if [[ -f "$filepath" ]]; then
        echo "  $filename already exists, skipping"
    else
        curl -fL "$WHISPER_BASE_URL/$filename" -o "$filepath" || {
            echo "Failed to download $filename" >&2
            exit 1
        }
    fi

    echo "Done!"
    ls -lh "$filepath"
}

ENGINE="${1:-moonshine-tiny}"

case "$ENGINE" in
    moonshine-tiny)
        download_moonshine_tiny
        ;;
    moonshine-base)
        download_moonshine_base
        ;;
    whisper-tiny)
        download_whisper_model "tiny" "ggml-tiny.en.bin"
        ;;
    whisper-base)
        download_whisper_model "base" "ggml-base.en.bin"
        ;;
    whisper-small)
        download_whisper_model "small" "ggml-small.en.bin"
        ;;
    whisper-medium)
        download_whisper_model "medium" "ggml-medium.en.bin"
        ;;
    whisper-large-v3)
        download_whisper_model "large-v3" "ggml-large-v3.bin"
        ;;
    whisper-large-v3-turbo)
        download_whisper_model "large-v3-turbo" "ggml-large-v3-turbo.bin"
        ;;
    all)
        download_moonshine_tiny
        download_moonshine_base
        download_whisper_model "tiny" "ggml-tiny.en.bin"
        download_whisper_model "base" "ggml-base.en.bin"
        download_whisper_model "small" "ggml-small.en.bin"
        download_whisper_model "medium" "ggml-medium.en.bin"
        download_whisper_model "large-v3" "ggml-large-v3.bin"
        download_whisper_model "large-v3-turbo" "ggml-large-v3-turbo.bin"
        ;;
    *)
        echo "Unknown engine: $ENGINE" >&2
        echo "Usage: $0 [moonshine-tiny|moonshine-base|whisper-tiny|whisper-base|whisper-small|whisper-medium|whisper-large-v3|whisper-large-v3-turbo|all]" >&2
        exit 1
        ;;
esac
