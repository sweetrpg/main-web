#!/bin/bash

set -e

ns=${1:-sweetrpg-main}

scriptdir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

pushd ${scriptdir}

echo "Deleting old secrets..."
kubectl delete -n "${ns}" secret template-credentials sweetrpg-registry web-files web-cache web-auth web-misc web-common || true

echo "Git credentials for template copying..."
kubectl create -n "${ns}" secret generic template-credentials \
    --from-env-file=../git-templates.env

echo "Docker registry config..."
kubectl create -n "${ns}" secret docker-registry sweetrpg-registry \
    --docker-server=registry.sweetrpg.com \
    --docker-username=docker \
    --docker-password=ESU7PnNtlt07zkvbnSyByrTdzllajxIQWqY7mswQR78
#kubectl create -n "${ns}" secret dockerconfigjson sweetrpg-registry \
#    --from-file=.dockerconfigjson

echo "NewRelic and logging config..."
kubectl create -n "${ns}" secret generic web-files \
    --from-file=newrelic.ini

echo "Other secrets..."
kubectl create -n "${ns}" secret generic web-cache \
    --from-env-file=cache.env
kubectl create -n "${ns}" secret generic web-auth \
    --from-env-file=auth.env
kubectl create -n "${ns}" secret generic web-misc \
    --from-env-file=misc.env
#kubectl create -n "${ns}" secret generic web-stripe \
#    --from-env-file=stripe.env
kubectl create -n "${ns}" secret generic web-common \
    --from-env-file=../common.env

popd
