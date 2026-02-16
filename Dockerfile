FROM python:3.12-slim

WORKDIR /app

COPY server/requirements-server.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

COPY server/ server/

RUN python -c "from server.main import app; print('[build] server.main imports OK')"

EXPOSE 8000

CMD ["sh", "-c", "exec uvicorn server.main:app --host 0.0.0.0 --port ${PORT:-8000} --log-level info"]
