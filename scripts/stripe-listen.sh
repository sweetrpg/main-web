#!/bin/bash

set -x
set -e
set -o pipefail

env_file=$1

export $(cat $env_file | xargs)

stripe listen --forward-to localhost:5000/billing/stripe/webhook
