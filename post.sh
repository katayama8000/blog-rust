#!/bin/bash

if [ "$#" -ne 2 ]; then
    echo "Error: Invalid number of arguments. Usage: $0 <title> <file>"
    exit 1
fi

title="$1"
file="posts/$2.md"

cargo run --bin markd "$title" "$file"
