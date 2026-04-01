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

# Use actual workspace path — support both Windows (WSL) and Linux
# In WSL, /mnt/c/Users/... maps to C:/Users/...; otherwise use env or default
import os as _os
_windows_path = "C:/Users/Zwmar/.openclaw/workspace"
if _os.path.exists(_windows_path):
    WORKSPACE = Path(_windows_path)
else:
    WORKSPACE = Path(_os.environ.get("OPENCLAW_WORKSPACE", "/home/zach/.openclaw/workspace"))

# Default projects dir — try the actual dev/Claw/projects path first
_default_projects = Path("/home/zach/.openclaw/workspace/dev/Claw/projects")
PROJECTS_DIR = Path(_os.environ.get("NOVA_PROJECTS_DIR", str(_default_projects if _default_projects.exists() else WORKSPACE.parent / "projects")))


def get_personality_state() -> dict:
    """Read current personality activations from persona module.
    
    The core.persona module lives at WORKSPACE/dev/.openclaw/workspace/core/persona.py
    (or WORKSPACE/core/persona.py on some setups). We try both.
    """
    import importlib
    
    for persona_path in [
        WORKSPACE / "dev" / ".openclaw" / "workspace",
        WORKSPACE,
    ]:
        if not persona_path.exists():
            continue
        sys.path.insert(0, str(persona_path))
        try:
            import core.persona as persona_module
            importlib.reload(persona_module)  # ensure fresh state
            persona = persona_module.get_persona()
            traits = persona.voice(5)
            # If traits are all 0 (fresh persona), derive from anchors for display
            if not traits:
                traits = _derive_traits_from_anchors(persona_module.ANCHORS, persona_module.TRAITS)
            return {
                "traits": traits,
                "anchors": persona_module.ANCHORS,
                "trait_definitions": persona_module.TRAITS,
            }
        except Exception:
            continue
        finally:
            sys.path.pop(0)

    return {"traits": [], "anchors": {}, "trait_definitions": {}}


def _derive_traits_from_anchors(anchors: dict, trait_defs: dict) -> list:
    """When persona hasn't heard any messages yet, show baseline traits from anchors."""
    # Start all traits at a small baseline, then nudge based on anchor signals
    traits = {t: 0.15 for t in trait_defs}  # faint baseline
    
    # Map anchors to trait influences
    anchor_trait_nudges = {
        "Zach_is_close":          [("warm", 0.6), ("candid", 0.4)],
        "I_am_allowed_disagree": [("candid", 0.5), ("direct", 0.3)],
        "I_have_opinions":        [("sharp", 0.5), ("candid", 0.4)],
        "I_am_resourceful":      [("direct", 0.4), ("sharp", 0.3)],
        "We_are_casual":         [("sardonic", 0.3), ("warm", 0.3)],
        "Respect_his_intelligence": [("direct", 0.4), ("contemplative", 0.3)],
    }
    
    for anchor_name, strength in anchors.items():
        nudges = anchor_trait_nudges.get(anchor_name, [])
        for trait, nudge in nudges:
            if trait in traits:
                traits[trait] += strength * nudge
    
    # Normalize and cap
    for t in traits:
        traits[t] = min(1.0, traits[t])
    
    ranked = sorted(traits.items(), key=lambda x: -x[1])
    return [
        (name, round(strength, 2), trait_defs.get(name, ""))
        for name, strength in ranked
        if strength > 0.1
    ][:5]


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
    projects_dir = PROJECTS_DIR
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
    """Check if gateway is reachable via its health endpoint."""
    try:
        r = requests.get(f"{GATEWAY_URL}/health", timeout=5)
        data = r.json()
        return {"online": data.get("ok", r.ok), "status": r.status_code, "detail": data.get("status", "")}
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
    Also updates Nova's personality state by feeding the message through the persona.
    """
    # Feed the message through the persona to update trait activations
    _persona_guide = ""
    try:
        import importlib
        for persona_path in [
            WORKSPACE / "dev" / ".openclaw" / "workspace",
            WORKSPACE,
        ]:
            if not persona_path.exists():
                continue
            sys.path.insert(0, str(persona_path))
            try:
                import core.persona as persona_module
                importlib.reload(persona_module)
                # Activate traits based on the incoming message
                _persona_guide = persona_module.on_message(req.message)
                break
            except Exception:
                continue
            finally:
                sys.path.pop(0)
    except Exception:
        pass

    # Get personality context (now with updated trait activations)
    try:
        personality = get_personality_state()
    except Exception:
        personality = {"traits": [], "anchors": {}}

    # Build personality context string from the guide (or fallback to traits)
    personality_context = _persona_guide if _persona_guide else ""
    if not personality_context:
        trait_lines = []
        for name, strength, desc in personality.get("traits", []):
            if strength > 0.2:
                level = "strongly" if strength > 0.6 else "somewhat" if strength > 0.3 else "faintly"
                trait_lines.append(f"- {name}: {desc} ({level})")
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
                finally:
                    # Decay personality after response completes.
                    # Run silently — do NOT let decay errors bubble up and
                    # shadow whatever exception (or normal completion) is active.
                    try:
                        for persona_path in [
                            WORKSPACE / "dev" / ".openclaw" / "workspace",
                            WORKSPACE,
                        ]:
                            if not persona_path.exists():
                                continue
                            sys.path.insert(0, str(persona_path))
                            try:
                                import core.persona as persona_module
                                persona_module.after_response()
                                break
                            except Exception:
                                continue
                            finally:
                                sys.path.pop(0)
                    except Exception:
                        pass  # swallow — decay is best-effort

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
        finally:
            # Decay personality after response completes
            try:
                for persona_path in [
                    WORKSPACE / "dev" / ".openclaw" / "workspace",
                    WORKSPACE,
                ]:
                    if not persona_path.exists():
                        continue
                    sys.path.insert(0, str(persona_path))
                    try:
                        import core.persona as persona_module
                        persona_module.after_response()
                        break
                    except Exception:
                        continue
                    finally:
                        sys.path.pop(0)
            except Exception:
                pass


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
