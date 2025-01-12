#!/bin/bash

if [ $# = 1 ]; then
  RUNS="$1"
else
  RUNS=-1
fi

mkdir -p corpus
cp tests/samples/*.blp corpus
cp tests/sample_errors/*.blp corpus
python3 tests/fuzz.py --runs $RUNS corpus