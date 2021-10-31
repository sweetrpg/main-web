#!/bin/bash

set -x
set -e
set -o pipefail

#export $(cat configs/dev.env | xargs)

pushd src

python3 appserver.py

popd
