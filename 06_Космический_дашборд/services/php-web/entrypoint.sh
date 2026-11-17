#!/usr/bin/env bash
set -eu
APP_DIR=/var/www/html
rsync -a --delete --exclude=.env --exclude=storage /opt/app/ "$APP_DIR/"
mkdir -p "$APP_DIR/storage/framework/cache/data" "$APP_DIR/storage/framework/sessions" "$APP_DIR/storage/framework/views" "$APP_DIR/storage/logs"
if [ ! -f "$APP_DIR/.env" ]; then
    cp "$APP_DIR/.env.example" "$APP_DIR/.env"
    php "$APP_DIR/artisan" key:generate --force
fi
chown -R www-data:www-data "$APP_DIR/storage" "$APP_DIR/bootstrap/cache"
exec php-fpm -F
