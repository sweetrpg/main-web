#!/bin/bash

set -x
set -e
set -o pipefail

env_file=$1

export $(cat $env_file | xargs)

stripe login --api-key "$STRIPE_API_KEY"
