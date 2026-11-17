#!/bin/sh
set -eu
mkdir -p /data/csv
printf '%s cd /app && python legacy.py > /proc/1/fd/1 2> /proc/1/fd/2\n' "${LEGACY_CRON:-*/5 * * * *}" > /etc/crontabs/root
exec crond -f -l 2
