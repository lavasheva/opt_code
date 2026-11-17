FROM python:3.12-alpine
RUN apk add --no-cache postgresql-client tzdata
ENV TZ=UTC
WORKDIR /app
COPY legacy.py entrypoint.sh ./
RUN chmod +x entrypoint.sh
ENTRYPOINT ["/app/entrypoint.sh"]
