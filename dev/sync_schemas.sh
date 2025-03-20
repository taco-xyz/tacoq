#!/bin/bash

# This script syncs the Avro schema in the client SDKs and the relay, copying the
# schema to the target directories.

SCHEMA_SOURCE="schemas/avro"

# Define target directories where the schema should be copied
TARGET_DIRS=(
  "client_sdks/python/tacoq/core/models/"
  "server/relay/src/models/"
)

echo "Syncing Avro schema to services..."

# Loop through target directories and copy the schema
for dir in "${TARGET_DIRS[@]}"; do
  mkdir -p "$dir"  # Ensure the directory exists
  cp "$SCHEMA_SOURCE" "$dir"
  echo "✅ Copied schema to $dir"
done

echo "✅ Schema sync complete."
