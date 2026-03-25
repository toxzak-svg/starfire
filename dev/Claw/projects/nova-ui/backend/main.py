"""
Nova UI — Backend
FastAPI app that connects to the OpenClaw gateway via HTTP.
Handles SSE parsing and clean streaming to the frontend.
"""

import os
import json
import re
import sys
from pathlib import Path
from typing import Optional

import requests
import uvicorn
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, StreamingResponse
from pydantic import BaseModel

# -------------------------------------------------------------------
# CONFIG
# -------------------------------------------------------------------

GATEWAY_URL = os.environ.get("OPENCLAW_GATEWAY_URL", "http://127.0.0.1:18789")
GATEWAY_TOKEN = os.environ.get("OPENCLAW_GATEWAY_TOKEN", "05a9121e8d27e2ca7f90ec7bab2ddd6b1b111b761aac0fc0")
UI_PORT = 8765

HEADERS = {
    "Authorization": f"Bearer {GATEWAY_TOKEN}",
    "Content-Type": "application/json",
}

# -------------------------------------------------------------------
# WORKSPACE STATE
# -------------------------------------------------------------------

WORKSPACE = Path("C:/Users/Zwmar/.openclaw/workspace")


def get_personality_state() -> dict:
    """Read current personality activations from persona module."""
    try:
        sys.path.insert(0, str(WORKSPACE))
        from importlib import import_module
        import core.persona as persona_module
        persona = persona_module.get_persona()
        return {
            "traits": persona.voice(5),
            "anchors": persona.anchors,
        }
    except Exception as e:
        return {"traits": [], "anchors": {}, "error": str(e)}


def get_context() -> dict:
    """Read session context and today's memory."""
    from datetime import datetime
    today = datetime.now().strftime("%Y-%m-%d")

    ctx_file = WORKSPACE / "CONTEXT.md"
    mem_file = WORKSPACE / "memory" / f"{today}.md"

    ctx = ""
    mem = ""

    try:
        ctx = ctx_file.read_text(encoding="utf-8", errors="ignore")[:3000] if ctx_file.exists() else ""
    except Exception:
        pass

    try:
        mem = mem_file.read_text(encoding="utf-8", errors="ignore")[-3000:] if mem_file.exists() else ""
    except Exception:
        pass

    return {"context": ctx, "memory": mem}


def get_projects() -> list:
    """List active projects."""
    projects_dir = Path("C:/Users/Zwmar/Claw/projects")
    if not projects_dir.exists():
        return []

    results = []
    for d in projects_dir.iterdir():
        if not d.is_dir() or d.name.startswith("."):
            continue
        readme = d / "README.md"
        try:
            desc = readme.read_text(encoding="utf-8", errors="ignore").split("\n")[0][:100] if readme.exists() else ""
        except Exception:
            desc = ""
        results.append({"name": d.name, "description": desc})
    return results


def get_gateway_health() -> dict:
    """Check if gateway is reachable."""
    try:
        r = requests.get(GATEWAY_URL, timeout=5)
        return {"online": True, "status": r.status_code}
    except Exception as e:
        return {"online": False, "error": str(e)}


# -------------------------------------------------------------------
# FASTAPI APP
# -------------------------------------------------------------------

app = FastAPI(title="Nova UI", version="1.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# -------------------------------------------------------------------
# MODELS
# -------------------------------------------------------------------

class ChatRequest(BaseModel):
    message: str
    stream: bool = True


# -------------------------------------------------------------------
# SSE PARSING — extract text deltas from gateway SSE
# -------------------------------------------------------------------

def parse_sse_events(chunk: bytes) -> list:
    """
    Parse SSE data from gateway.
    Yields dicts like: {"type": "delta", "text": "..."} or {"type": "status", "text": "..."}
    """
    try:
        text = chunk.decode("utf-8", errors="replace")
    except Exception:
        return []

    events = []
    for line in text.split("\n"):
        line = line.strip()
        if not line:
            continue
        if line.startswith("event:"):
            continue
        if line.startswith("data:"):
            data = line[5:].strip()
            try:
                obj = json.loads(data)
                # Extract text delta
                if obj.get("type") == "response.output_text.delta":
                    delta = obj.get("delta", "")
                    if delta:
                        events.append({"type": "delta", "text": delta})
                elif obj.get("type") == "response.completed":
                    events.append({"type": "done"})
                elif obj.get("type") == "response.created":
                    events.append({"type": "created", "id": obj.get("response", {}).get("id", "")})
            except Exception:
                pass
    return events


# -------------------------------------------------------------------
# ROUTES
# -------------------------------------------------------------------

@app.get("/")
async def root():
    """Serve the UI."""
    static_dir = Path(__file__).parent.parent / "frontend"
    index_file = static_dir / "index.html"
    if index_file.exists():
        return FileResponse(str(index_file), media_type="text/html")
    raise HTTPException(status_code=404, detail="UI not found")


@app.get("/api/status")
async def status():
    """Overall system status."""
    health = get_gateway_health()
    try:
        context = get_context()
        projects = get_projects()
    except Exception:
        context = {"context": "", "memory": ""}
        projects = []

    return {
        "gateway": health,
        "context": context,
        "projects": projects,
        "ui_version": "1.0.0",
    }


@app.post("/api/chat")
async def chat(req: ChatRequest):
    """
    Send a message to Nova via the gateway /v1/responses endpoint.
    Parses SSE from gateway and streams clean text deltas to frontend.
    """
    # Get personality context
    try:
        personality = get_personality_state()
    except Exception:
        personality = {"traits": [], "anchors": {}}

    # Build personality context string
    trait_lines = []
    for name, strength, desc in personality.get("traits", []):
        if strength > 0.2:
            level = "strongly" if strength > 0.6 else "somewhat" if strength > 0.3 else "faintly"
            trait_lines.append(f"- {name}: {desc} ({level})")

    personality_context = ""
    if trait_lines:
        personality_context = (
            "\n\n[Internal voice shaping — Nova is informed by these active traits:\n"
            + "\n".join(trait_lines)
            + "\nLet these inform your voice naturally. Do not announce them.]\n"
        )

    full_message = personality_context + req.message

    # Build the OpenResponses request
    payload = {
        "model": "openclaw:main",
        "input": [
            {"type": "message", "role": "user", "content": full_message}
        ],
        "stream": req.stream,
    }

    gateway_url = f"{GATEWAY_URL}/v1/responses"
    headers = {
        "Authorization": f"Bearer {GATEWAY_TOKEN}",
        "Content-Type": "application/json",
    }

    if req.stream:
        # Stream: parse SSE and forward clean text deltas
        try:
            r = requests.post(
                gateway_url,
                json=payload,
                headers=headers,
                stream=True,
                timeout=120,
            )
            if r.status_code != 200:
                raise HTTPException(status_code=r.status_code, detail=f"Gateway: {r.text[:200]}")

            async def event_generator():
                buffer = ""
                try:
                    for chunk in r.iter_content(chunk_size=None):
                        if chunk:
                            buffer += chunk.decode("utf-8", errors="replace")

                            # Find complete lines
                            while "\n" in buffer:
                                newline_idx = buffer.index("\n")
                                line = buffer[:newline_idx].strip()
                                buffer = buffer[newline_idx + 1:]

                                if not line or not line.startswith("data:"):
                                    continue

                                data_str = line[5:].strip()
                                if not data_str:
                                    continue

                                try:
                                    obj = json.loads(data_str)
                                except Exception:
                                    continue

                                # Text delta → forward as simple text event
                                if obj.get("type") == "response.output_text.delta":
                                    delta = obj.get("delta", "")
                                    if delta:
                                        yield f"data: {json.dumps({'type': 'text', 'content': delta})}\n\n".encode("utf-8")

                                # Done → send done signal
                                elif obj.get("type") == "response.completed":
                                    yield b"data: {\"type\": \"done\"}\n\n"

                except Exception as e:
                    yield f"data: {json.dumps({'type': 'error', 'content': str(e)})}\n\n".encode("utf-8")

            return StreamingResponse(
                event_generator(),
                media_type="text/event-stream",
                headers={"X-Accel-Buffering": "no"},
            )

        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))

    else:
        # Non-streaming
        try:
            r = requests.post(gateway_url, json=payload, headers=headers, timeout=120)
            data = r.json()

            # Extract text from response
            output = data.get("output", [])
            text = ""
            for item in output:
                if item.get("type") == "message":
                    for content in item.get("content", []):
                        if content.get("type") == "output_text":
                            text += content.get("text", "")

            return {"type": "text", "content": text}
        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/personality")
async def personality():
    """Get current personality state."""
    return get_personality_state()


@app.get("/api/context")
async def context():
    """Get current context."""
    return get_context()


@app.get("/api/projects")
async def projects():
    """Get active projects."""
    return get_projects()


# -------------------------------------------------------------------
# ENTRY POINT
# -------------------------------------------------------------------

def main():
    print(f"\n{'='*50}")
    print(f"  Nova UI — http://localhost:{UI_PORT}")
    print(f"  Gateway: {GATEWAY_URL}")
    print(f"{'='*50}\n")
    uvicorn.run(app, host="127.0.0.1", port=UI_PORT, log_level="info")


if __name__ == "__main__":
    main()
