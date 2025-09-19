#!/bin/sh

# seta o usuário do banco de dados pois o padrão é root
export USER="app_user"
# Executa as migrações do banco de dados
poetry run python manage.py migrate
# Executa as migrações do banco de dados
# poetry run alembic upgrade head

SSL_CERT_FILE="./certs/app.fomento.to.gov.br.crt"
SSL_KEY_FILE="./certs/app.fomento.to.gov.br.key"

# Inicia a aplicação
poetry run uvicorn --host 0.0.0.0 --port 8004 --workers 10 app.main:app --log-level error

#python manage.py migrate
#python run uvicorn --host 0.0.0.0 --port 8004 --workers 10 app.main:app


# Inicia a aplicação com HTTPS
#poetry run uvicorn --host 0.0.0.0 --port 8004 --workers 10 \
#    --ssl-certfile $SSL_CERT_FILE \
#    --ssl-keyfile $SSL_KEY_FILE \
#    app.main:app --log-level error
