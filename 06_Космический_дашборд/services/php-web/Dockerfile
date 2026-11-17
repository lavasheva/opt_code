FROM php:8.3-fpm-alpine

# пакеты и расширения
RUN apk add --no-cache bash curl git unzip rsync postgresql-dev oniguruma-dev libxml2-dev curl-dev \
 && docker-php-ext-install pdo pdo_pgsql mbstring dom xml xmlwriter curl

# composer
COPY --from=composer:2 /usr/bin/composer /usr/bin/composer

# каталоги
RUN mkdir -p /var/www/html /opt/laravel-patches
WORKDIR /var/www/html

RUN composer create-project --no-interaction --prefer-dist --no-install --no-scripts laravel/laravel:^12 /opt/app
COPY composer.json composer.lock /opt/app/
RUN cd /opt/app && composer install --no-interaction --prefer-dist
COPY laravel-patches /opt/app/

# entrypoint
COPY entrypoint.sh /usr/local/bin/php_entrypoint.sh
RUN sed -i 's/\r$//' /usr/local/bin/php_entrypoint.sh && chmod +x /usr/local/bin/php_entrypoint.sh

CMD ["bash", "/usr/local/bin/php_entrypoint.sh"]
