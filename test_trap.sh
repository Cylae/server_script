#!/bin/bash
set -euo pipefail

func_a() {
  local myvar="A_VAR"
  # Use double quotes to bake the value in, like in the real code
  trap "echo returning A with $myvar" RETURN
}

func_b() {
  func_a
  echo "inside B"
}

echo "Calling B"
func_b
echo "Finished B"
