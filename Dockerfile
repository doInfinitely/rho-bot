FROM python:3.12-slim

WORKDIR /app

# 1. Install dependencies (cached until requirements change)
COPY server/requirements-server.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# 2. Copy application code (separate layer so code changes don't re-install deps)
COPY server/ server/

# Verify the code is actually there
RUN python -c "from server.main import app; print('[build] server.main imports OK')"

EXPOSE 8000

CMD ["sh", "-c", "exec uvicorn server.main:app --host 0.0.0.0 --port ${PORT:-8000} --log-level info"]
