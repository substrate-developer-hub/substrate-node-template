#!/bin/bash

# script to check if file size of fmt.log is 0

# Fail fast if any commands exists with error
set -e

# Print all executed commands
set -x

if [[ ! -s fmt.log ]]; then
    exit 0
fi

exit 1
