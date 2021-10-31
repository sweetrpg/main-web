#!/bin/bash

set -x
set -e
set -o pipefail

# python3 -m venv venv
conda create --name sweetrpg python=3.8
conda activate sweetrpg
# venv/bin/activate
