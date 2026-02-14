# rho-bot

An autonomous desktop agent powered by **hierarchical goal induction**. It observes how you use your computer, induces goals at multiple levels of abstraction, and learns to act on your behalf.

## Architecture

```
rho_bot/
├── server/     FastAPI backend – WebSocket agent endpoint, REST dashboard API, model inference
├── desktop/    Tauri 2 macOS app – captures context, executes predicted actions
└── website/    Next.js – landing page at rho-bot.net + authenticated dashboard
```

### Data Flow

1. **Desktop client** captures screenshots (CoreGraphics), reads the accessibility tree (AXUIElement), and logs input events (CGEvent tap).
2. Context bundles are streamed to the **server** over a persistent WebSocket connection.
3. The server runs the context through the **model pipeline** (PyTorch / JAX) and returns an action prediction.
4. The desktop client **executes** the action by posting synthetic CGEvent mouse/keyboard events.
5. The **website dashboard** shows agent status, session history, and action logs via a REST API.

---

## Quick Start

### 1. Server

```bash
cd server
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

# Copy and edit environment variables
cp .env.example .env

# Run (requires PostgreSQL running on localhost:5432)
uvicorn server.main:app --reload --host 0.0.0.0 --port 8000
```

The server will create database tables on first startup. Visit `http://localhost:8000/docs` for the interactive API docs.

### 2. Website

```bash
cd website
npm install

# Copy and edit environment variables
cp .env.local.example .env.local

npm run dev
```

Open `http://localhost:3000`. Sign up for an account, then copy the auth token from the dashboard.

### 3. Desktop Client

Prerequisites: [Rust](https://rustup.rs) and [Node.js](https://nodejs.org).

```bash
cd desktop
npm install
npm run tauri dev
```

On first run, macOS will prompt for **Accessibility** and **Screen Recording** permissions (System Settings > Privacy & Security). Grant both for full functionality.

Paste your auth token from the website dashboard into the desktop client's Settings tab. Click **Start Agent** to begin the observe-predict-execute loop.

---

## Server API

| Endpoint | Method | Description |
|---|---|---|
| `/health` | GET | Health check |
| `/auth/signup` | POST | Create account, returns JWT |
| `/auth/login` | POST | Authenticate, returns JWT |
| `/ws/agent` | WebSocket | Desktop client agent connection |
| `/api/me` | GET | Current user info |
| `/api/agent/status` | GET | Agent online/offline status |
| `/api/sessions` | GET | List sessions |
| `/api/sessions/{id}/actions` | GET | Actions in a session |

## Model Integration

Edit `server/services/model_service.py` to wire in your trained models:

```python
class ModelService:
    def __init__(self, model_path: str):
        self._model = torch.load(model_path)

    async def predict_action(self, context: ContextPayload) -> ActionPayload:
        preprocessed = self.preprocess(context)
        raw = self._model(preprocessed)
        return self.postprocess(raw)
```

Set `RHOBOT_MODEL_PATH` in your `.env` to the checkpoint path.

---

## License

Proprietary. All rights reserved.
