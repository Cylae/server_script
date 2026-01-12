#!/bin/bash
set -euo pipefail

func() {
    local myvar="hello"
    trap 'echo "Cleaning $myvar"' RETURN
    echo "Inside func"
}

func
