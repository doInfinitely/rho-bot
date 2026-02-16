FROM python:3.12-slim

WORKDIR /app

# Install server dependencies (no torch -- inference runs on Modal)
COPY server/requirements-server.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

COPY server/ server/

# Bust Docker cache on each deploy
ARG RAILWAY_GIT_COMMIT_SHA=unknown
RUN echo "build: ${RAILWAY_GIT_COMMIT_SHA}" > /app/.build-id

EXPOSE 8000

CMD ["sh", "-c", "echo 'Starting server on port ${PORT:-8000}' && exec uvicorn server.main:app --host 0.0.0.0 --port ${PORT:-8000} --log-level info"]
