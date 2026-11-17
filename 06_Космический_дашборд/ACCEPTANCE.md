# Приёмочные сценарии

Сначала выполнить `setup.ps1` в этой папке. Нужен работающий Docker Desktop.

1. `docker compose ps`: БД healthy, rust_iss/php/nginx/pascal_legacy запущены.
2. `http://localhost:8081/health`: status=ok и UTC now.
3. `http://localhost:8080/dashboard`: карта, метрики, JWST-галерея, события; нет ошибок шаблонов. Сохранить реальный скриншот.
4. `http://localhost:8080/osdr`: таблица или понятное сообщение при пустом источнике.
5. `http://localhost:8080/page/welcome`: читаемая CMS-страница.
6. `http://localhost:8080/page/unsafe`: HTML отформатирован, script удалён, код не выполняется.
7. `http://localhost:8080/api/astro/events?lat=100`: HTTP 200, ok=false, INVALID_COORDINATES, trace_id.
8. Без Astronomy-ключей запрос events: HTTP 200, MISSING_CREDENTIALS. С рабочими ключами — исходная структура успешного ответа.
9. Через тестовый HTTP-источник вернуть 403 для OSDR: `/osdr/sync` отвечает HTTP 200, ok=false, UPSTREAM_403. Не повторяет 403.
10. Дважды синхронизировать один dataset_id: число строк не растёт, данные обновляются.
11. Одновременно вызвать ручную синхронизацию и фоновую задачу: advisory lock не даёт выполнять одинаковую задачу параллельно.
12. `/space/summary`: ключи apod, neo, flr, cme, spacex, iss, osdr_count. Сравнить структуры с исходником на одинаковом наполнении БД.
13. `docker compose logs pascal_legacy`: cron и успешный COPY, в CSV один заголовок и одна строка данных.
14. `docker compose exec --user www-data php php artisan test --filter=SpaceClientTest`: кэширует успех, не кэширует ошибку, валидирует координаты и очищает CMS HTML. Тесты запускаются от веб-пользователя, чтобы не создавать файлы кэша с владельцем root.
15. `docker compose run --rm --build rust_tests`: модульные проверки доменных функций и формата ошибок.

Проверки 3, 4 и успешные запросы провайдеров требуют сети, актуальных ключей и доступного внешнего API. Не заменять фактические скриншоты макетами. После прогона сохранить `docker compose logs --no-color` в `logs.txt`, исключив секреты.
