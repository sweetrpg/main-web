#!/bin/bash

set -e

#newrelic-admin run-program gunicorn wsgi:app
# newrelic-admin run-program uwsgi --http :${PORT} --module wsgi:app --master --processes 4 --threads 2
uwsgi --http :${PORT} --module wsgi:app --master --processes 4 --threads 2
