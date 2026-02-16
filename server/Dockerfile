FROM python:3.12-slim

WORKDIR /app

# Install server dependencies (no torch -- inference runs on Modal)
COPY server/requirements-server.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

COPY server/ server/

EXPOSE 8000

CMD uvicorn server.main:app --host 0.0.0.0 --port ${PORT:-8000}
