#!/usr/bin/env python3
"""
Screen Error Solver — Real-time screen watcher that detects errors and offers solutions.

Usage:
    python screen_watcher.py                           # Default config
    python screen_watcher.py --config config.yaml      # Custom config
    python screen_watcher.py --provider gemini          # Use Gemini API
    python screen_watcher.py --provider ollama          # Use local Ollama
    python screen_watcher.py --region 0,400,1920,680    # Watch specific region

Environment Variables:
    GEMINI_API_KEY    — Google Gemini API key
    OPENAI_API_KEY    — OpenAI API key
    ANTHROPIC_API_KEY — Anthropic API key
"""

import argparse
import base64
import hashlib
import io
import json
import logging
import os
import re
import sys
import time
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional

import mss
import mss.tools
from PIL import Image

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

DEFAULT_CONFIG = {
    "capture": {
        "interval_seconds": 2,
        "monitor": 0,
        "region": None,
        "downscale_factor": 0.5,
    },
    "detection": {
        "mode": "hybrid",  # "vision", "ocr", "hybrid"
        "vision_model": "gemini-2.0-flash",
        "ocr_confidence_threshold": 60,
        "error_cooldown_seconds": 30,
        "frame_diff_threshold": 0.05,  # 5% pixel change to trigger analysis
    },
    "solution": {
        "model": "gemini-2.0-flash",
        "max_tokens": 1024,
        "include_context_lines": 20,
    },
    "delivery": {
        "method": "notification",  # "notification", "terminal", "file", "webhook"
        "webhook_url": None,
        "log_file": "./error_solutions.log",
        "sound": True,
    },
    "api": {
        "provider": "gemini",  # "openai", "gemini", "anthropic", "ollama"
        "api_key_env": "GEMINI_API_KEY",
        "ollama_base_url": "http://localhost:11434",
    },
}

# ---------------------------------------------------------------------------
# Error Patterns (OCR-based detection)
# ---------------------------------------------------------------------------

ERROR_PATTERNS = [
    # Python
    (r"Traceback \(most recent call last\)", "python_traceback", "critical"),
    (r"(\w+Error): (.+)", "python_error", "critical"),
    (r"(\w+Exception): (.+)", "python_exception", "critical"),
    (r"(\w+Warning): (.+)", "python_warning", "warning"),
    (r"SyntaxError: (.+)", "syntax_error", "critical"),
    (r"ModuleNotFoundError: No module named '(.+)'", "import_error", "critical"),
    # JavaScript / TypeScript
    (r"(TypeError|ReferenceError|SyntaxError): (.+)", "js_error", "critical"),
    (r"Uncaught (\w+Error): (.+)", "js_uncaught", "critical"),
    (r"error TS(\d+): (.+)", "typescript_error", "critical"),
    (r"ERR_MODULE_NOT_FOUND", "node_module_error", "critical"),
    (r"Cannot find module '(.+)'", "node_import_error", "critical"),
    # Rust
    (r"error\[E(\d+)\]: (.+)", "rust_error", "critical"),
    (r"panicked at (.+)", "rust_panic", "critical"),
    # Build systems
    (r"BUILD FAILED", "build_failure", "critical"),
    (r"FAILED|FAILURE", "generic_failure", "critical"),
    (r"make: \*\*\* .+ Error \d+", "make_error", "critical"),
    (r"npm ERR!", "npm_error", "critical"),
    (r"cargo error", "cargo_error", "critical"),
    # Terminal / Shell
    (r"command not found", "command_not_found", "critical"),
    (r"No such file or directory", "file_not_found", "critical"),
    (r"Permission denied", "permission_denied", "critical"),
    (r"segmentation fault", "segfault", "critical"),
    # Network / HTTP
    (r"(4\d{2}) ([\w\s]+)", "http_client_error", "warning"),
    (r"(5\d{2}) ([\w\s]+)", "http_server_error", "critical"),
    (r"ECONNREFUSED", "connection_refused", "critical"),
    (r"ERR_CONNECTION_REFUSED", "connection_refused", "critical"),
    (r"fetch failed", "fetch_error", "critical"),
    (r"CORS", "cors_error", "warning"),
    # Docker
    (r"exited with code (\d+)", "docker_exit", "critical"),
    (r"OOMKilled", "oom_killed", "critical"),
    # GPU / CUDA
    (r"CUDA out of memory", "cuda_oom", "critical"),
    (r"RuntimeError: CUDA", "cuda_runtime", "critical"),
    (r"cuDNN error", "cudnn_error", "critical"),
    # Git
    (r"CONFLICT \(content\)", "git_conflict", "warning"),
    (r"fatal: (.+)", "git_fatal", "critical"),
    (r"error: failed to push", "git_push_error", "critical"),
]

COMPILED_PATTERNS = [(re.compile(p, re.IGNORECASE), name, sev) for p, name, sev in ERROR_PATTERNS]

# ---------------------------------------------------------------------------
# Data Classes
# ---------------------------------------------------------------------------

@dataclass
class DetectedError:
    """Represents a detected error from screen analysis."""
    timestamp: str
    error_type: str
    error_message: str
    severity: str  # "critical", "warning", "info"
    file_location: Optional[str] = None
    language: Optional[str] = None
    framework: Optional[str] = None
    context_summary: Optional[str] = None
    raw_text: Optional[str] = None
    screenshot_path: Optional[str] = None

    def hash_key(self) -> str:
        """Unique hash for deduplication."""
        return hashlib.md5(
            f"{self.error_type}:{self.error_message[:100]}".encode()
        ).hexdigest()


@dataclass
class Solution:
    """A generated solution for a detected error."""
    cause: str
    fix: str
    alternatives: list[str] = field(default_factory=list)
    confidence: float = 0.0


# ---------------------------------------------------------------------------
# Screen Capture
# ---------------------------------------------------------------------------

class ScreenCapture:
    """Captures screenshots using mss (cross-platform)."""

    def __init__(self, monitor: int = 0, region: Optional[dict] = None,
                 downscale: float = 0.5):
        self.monitor = monitor
        self.region = region
        self.downscale = downscale
        self._sct = mss.mss()
        self._prev_hash: Optional[str] = None

    def capture(self) -> tuple[Image.Image, bool]:
        """
        Capture a screenshot.
        Returns (image, has_changed) where has_changed indicates
        significant pixel difference from last frame.
        """
        if self.region:
            monitor = self.region
        else:
            monitors = self._sct.monitors
            if self.monitor < len(monitors):
                monitor = monitors[self.monitor]
            else:
                monitor = monitors[0]

        sct_img = self._sct.grab(monitor)
        img = Image.frombytes("RGB", sct_img.size, sct_img.bgra, "raw", "BGRX")

        # Downscale for efficiency
        if self.downscale < 1.0:
            new_size = (
                int(img.width * self.downscale),
                int(img.height * self.downscale),
            )
            img = img.resize(new_size, Image.LANCZOS)

        # Check if frame changed significantly
        current_hash = hashlib.md5(img.tobytes()[:10000]).hexdigest()
        has_changed = current_hash != self._prev_hash
        self._prev_hash = current_hash

        return img, has_changed

    def capture_as_base64(self) -> tuple[str, bool]:
        """Capture and return as base64-encoded PNG."""
        img, changed = self.capture()
        buffer = io.BytesIO()
        img.save(buffer, format="PNG", optimize=True)
        b64 = base64.b64encode(buffer.getvalue()).decode("utf-8")
        return b64, changed

    def save_capture(self, path: str) -> tuple[str, bool]:
        """Capture and save to file."""
        img, changed = self.capture()
        img.save(path, format="PNG")
        return path, changed

    def close(self):
        self._sct.close()


# ---------------------------------------------------------------------------
# OCR Engine
# ---------------------------------------------------------------------------

class OCREngine:
    """Extract text from screenshots using Tesseract OCR."""

    def __init__(self, confidence_threshold: int = 60):
        self.confidence_threshold = confidence_threshold
        self._available = False
        try:
            import pytesseract
            self._pytesseract = pytesseract
            # Test that tesseract binary is accessible
            pytesseract.get_tesseract_version()
            self._available = True
        except Exception:
            logging.warning(
                "pytesseract not available. Install with: "
                "pip install pytesseract && sudo apt install tesseract-ocr"
            )

    @property
    def available(self) -> bool:
        return self._available

    def extract_text(self, image: Image.Image) -> str:
        """Extract all text from image."""
        if not self._available:
            return ""
        try:
            return self._pytesseract.image_to_string(image)
        except Exception as e:
            logging.error(f"OCR failed: {e}")
            return ""

    def extract_with_positions(self, image: Image.Image) -> list[dict]:
        """Extract text with bounding box positions."""
        if not self._available:
            return []
        try:
            data = self._pytesseract.image_to_data(
                image, output_type=self._pytesseract.Output.DICT
            )
            results = []
            for i in range(len(data["text"])):
                conf = int(data["conf"][i])
                text = data["text"][i].strip()
                if conf >= self.confidence_threshold and text:
                    results.append({
                        "text": text,
                        "x": data["left"][i],
                        "y": data["top"][i],
                        "w": data["width"][i],
                        "h": data["height"][i],
                        "confidence": conf,
                    })
            return results
        except Exception as e:
            logging.error(f"OCR with positions failed: {e}")
            return []


# ---------------------------------------------------------------------------
# Error Detector
# ---------------------------------------------------------------------------

class ErrorDetector:
    """Detects errors using OCR patterns and/or vision LLM."""

    def __init__(self, config: dict):
        self.mode = config["detection"]["mode"]
        self.ocr = OCREngine(config["detection"]["ocr_confidence_threshold"])
        self.cooldown = config["detection"]["error_cooldown_seconds"]
        self._recent_errors: dict[str, float] = {}  # hash -> last_seen timestamp
        self._llm = LLMClient(config)

    def detect(self, image: Image.Image, b64_image: str) -> Optional[DetectedError]:
        """
        Run error detection on a captured frame.
        Returns DetectedError if an error is found, None otherwise.
        """
        if self.mode == "ocr":
            return self._detect_ocr(image)
        elif self.mode == "vision":
            return self._detect_vision(b64_image)
        else:  # hybrid
            result = self._detect_ocr(image)
            if result:
                return result
            # Fall back to vision for visual-only errors
            return self._detect_vision(b64_image)

    def _detect_ocr(self, image: Image.Image) -> Optional[DetectedError]:
        """Detect errors using OCR + regex pattern matching."""
        if not self.ocr.available:
            return None

        text = self.ocr.extract_text(image)
        if not text.strip():
            return None

        for pattern, error_name, severity in COMPILED_PATTERNS:
            match = pattern.search(text)
            if match:
                error_msg = match.group(0)

                # Extract surrounding context (lines around the match)
                lines = text.split("\n")
                match_line_idx = None
                for i, line in enumerate(lines):
                    if error_msg in line or pattern.search(line):
                        match_line_idx = i
                        break

                context_start = max(0, (match_line_idx or 0) - 5)
                context_end = min(len(lines), (match_line_idx or 0) + 10)
                context = "\n".join(lines[context_start:context_end])

                error = DetectedError(
                    timestamp=datetime.now().isoformat(),
                    error_type=error_name,
                    error_message=error_msg,
                    severity=severity,
                    raw_text=context,
                )

                # Check cooldown
                if self._is_duplicate(error):
                    return None

                return error

        return None

    def _detect_vision(self, b64_image: str) -> Optional[DetectedError]:
        """Detect errors by sending screenshot to a vision LLM."""
        try:
            result = self._llm.analyze_screenshot(b64_image)
            if not result or not result.get("has_error"):
                return None

            error = DetectedError(
                timestamp=datetime.now().isoformat(),
                error_type=result.get("error_type", "unknown"),
                error_message=result.get("error_message", ""),
                severity=result.get("severity", "warning"),
                file_location=result.get("file_location"),
                language=result.get("language"),
                framework=result.get("framework"),
                context_summary=result.get("context_summary"),
            )

            if self._is_duplicate(error):
                return None

            return error

        except Exception as e:
            logging.error(f"Vision detection failed: {e}")
            return None

    def _is_duplicate(self, error: DetectedError) -> bool:
        """Check if this error was recently detected (deduplication)."""
        key = error.hash_key()
        now = time.time()

        if key in self._recent_errors:
            if now - self._recent_errors[key] < self.cooldown:
                return True

        self._recent_errors[key] = now

        # Clean old entries
        self._recent_errors = {
            k: v for k, v in self._recent_errors.items()
            if now - v < self.cooldown * 2
        }
        return False


# ---------------------------------------------------------------------------
# LLM Client
# ---------------------------------------------------------------------------

class LLMClient:
    """Unified LLM client supporting multiple providers."""

    def __init__(self, config: dict):
        self.provider = config["api"]["provider"]
        self.vision_model = config["detection"]["vision_model"]
        self.solution_model = config["solution"]["model"]
        self.max_tokens = config["solution"]["max_tokens"]
        self.api_key = os.environ.get(config["api"].get("api_key_env", ""), "")
        self.ollama_url = config["api"].get("ollama_base_url", "http://localhost:11434")

    def analyze_screenshot(self, b64_image: str) -> Optional[dict]:
        """Send screenshot to vision LLM for error analysis."""
        prompt = (
            "Analyze this screenshot of a developer's screen. Your task:\n\n"
            "1. Is there an error, warning, or failure visible? (yes/no)\n"
            "2. If yes, identify:\n"
            "   - Error type (syntax, runtime, build, network, permission, dependency, etc.)\n"
            "   - The exact error message\n"
            "   - The file/line if visible\n"
            "   - The programming language/framework\n"
            "3. Provide a severity rating (critical/warning/info)\n\n"
            "Respond ONLY in JSON format:\n"
            '{"has_error": true/false, "error_type": "...", "error_message": "...", '
            '"file_location": "...", "language": "...", "framework": "...", '
            '"severity": "critical|warning|info", "context_summary": "..."}\n\n'
            'If no error is visible, respond with {"has_error": false}.'
        )

        if self.provider == "gemini":
            return self._gemini_vision(prompt, b64_image)
        elif self.provider == "openai":
            return self._openai_vision(prompt, b64_image)
        elif self.provider == "anthropic":
            return self._anthropic_vision(prompt, b64_image)
        elif self.provider == "ollama":
            return self._ollama_vision(prompt, b64_image)
        elif self.provider == "blackbox":
            return self._blackbox_vision(prompt, b64_image)
        else:
            raise ValueError(f"Unknown provider: {self.provider}")

    def generate_solution(self, error: DetectedError) -> Solution:
        """Generate a solution for a detected error."""
        prompt = (
            "You are an expert developer assistant. The user is actively coding and hit this error.\n"
            "Provide a concise, actionable fix.\n\n"
            f"## Error Context\n"
            f"- Error type: {error.error_type}\n"
            f"- Error message: {error.error_message}\n"
            f"- File: {error.file_location or 'unknown'}\n"
            f"- Language/Framework: {error.language or 'unknown'}/{error.framework or 'unknown'}\n"
            f"- What the user appears to be doing: {error.context_summary or 'unknown'}\n"
        )

        if error.raw_text:
            prompt += f"\n## Raw terminal/editor text around the error:\n```\n{error.raw_text[:2000]}\n```\n"

        prompt += (
            "\n## Instructions\n"
            "1. Explain what caused this error in ONE sentence\n"
            "2. Provide the exact fix (code snippet, command, or config change)\n"
            "3. If there are multiple possible causes, list the top 2-3 most likely\n"
            "4. Keep it under 200 words\n\n"
            "## Response Format (use exactly this):\n"
            "CAUSE: <one sentence>\n"
            "FIX: <code or command>\n"
            "ALT: <alternative fix if the first doesn't work>\n"
        )

        try:
            response = self._text_completion(prompt)
            return self._parse_solution(response)
        except Exception as e:
            logging.error(f"Solution generation failed: {e}")
            return Solution(
                cause="Could not generate solution",
                fix=f"Search for: {error.error_message}",
                alternatives=["Check the error message and stack trace manually"],
            )

    def _text_completion(self, prompt: str) -> str:
        """Get a text completion from the configured provider."""
        import requests

        if self.provider == "gemini":
            url = f"https://generativelanguage.googleapis.com/v1beta/models/{self.solution_model}:generateContent"
            resp = requests.post(
                url,
                params={"key": self.api_key},
                json={"contents": [{"parts": [{"text": prompt}]}]},
                timeout=30,
            )
            resp.raise_for_status()
            data = resp.json()
            return data["candidates"][0]["content"]["parts"][0]["text"]

        elif self.provider == "openai":
            url = "https://api.openai.com/v1/chat/completions"
            resp = requests.post(
                url,
                headers={"Authorization": f"Bearer {self.api_key}"},
                json={
                    "model": self.solution_model,
                    "messages": [{"role": "user", "content": prompt}],
                    "max_tokens": self.max_tokens,
                },
                timeout=30,
            )
            resp.raise_for_status()
            return resp.json()["choices"][0]["message"]["content"]

        elif self.provider == "anthropic":
            url = "https://api.anthropic.com/v1/messages"
            resp = requests.post(
                url,
                headers={
                    "x-api-key": self.api_key,
                    "anthropic-version": "2023-06-01",
                },
                json={
                    "model": self.solution_model,
                    "max_tokens": self.max_tokens,
                    "messages": [{"role": "user", "content": prompt}],
                },
                timeout=30,
            )
            resp.raise_for_status()
            return resp.json()["content"][0]["text"]

        elif self.provider == "ollama":
            url = f"{self.ollama_url}/api/generate"
            resp = requests.post(
                url,
                json={"model": self.solution_model, "prompt": prompt, "stream": False},
                timeout=60,
            )
            resp.raise_for_status()
            return resp.json()["response"]

        raise ValueError(f"Unknown provider: {self.provider}")

    def _gemini_vision(self, prompt: str, b64_image: str) -> Optional[dict]:
        import requests
        url = f"https://generativelanguage.googleapis.com/v1beta/models/{self.vision_model}:generateContent"
        payload = {
            "contents": [{
                "parts": [
                    {"text": prompt},
                    {"inline_data": {"mime_type": "image/png", "data": b64_image}},
                ]
            }]
        }
        resp = requests.post(url, params={"key": self.api_key}, json=payload, timeout=30)
        resp.raise_for_status()
        text = resp.json()["candidates"][0]["content"]["parts"][0]["text"]
        # Strip markdown code fences if present
        text = re.sub(r"```json\s*", "", text)
        text = re.sub(r"```\s*", "", text)
        return json.loads(text)

    def _openai_vision(self, prompt: str, b64_image: str) -> Optional[dict]:
        import requests
        url = "https://api.openai.com/v1/chat/completions"
        resp = requests.post(
            url,
            headers={"Authorization": f"Bearer {self.api_key}"},
            json={
                "model": self.vision_model,
                "messages": [{
                    "role": "user",
                    "content": [
                        {"type": "text", "text": prompt},
                        {"type": "image_url", "image_url": {
                            "url": f"data:image/png;base64,{b64_image}"
                        }},
                    ],
                }],
                "max_tokens": 500,
            },
            timeout=30,
        )
        resp.raise_for_status()
        text = resp.json()["choices"][0]["message"]["content"]
        text = re.sub(r"```json\s*", "", text)
        text = re.sub(r"```\s*", "", text)
        return json.loads(text)

    def _anthropic_vision(self, prompt: str, b64_image: str) -> Optional[dict]:
        import requests
        url = "https://api.anthropic.com/v1/messages"
        resp = requests.post(
            url,
            headers={
                "x-api-key": self.api_key,
                "anthropic-version": "2023-06-01",
            },
            json={
                "model": self.vision_model,
                "max_tokens": 500,
                "messages": [{
                    "role": "user",
                    "content": [
                        {"type": "image", "source": {
                            "type": "base64",
                            "media_type": "image/png",
                            "data": b64_image,
                        }},
                        {"type": "text", "text": prompt},
                    ],
                }],
            },
            timeout=30,
        )
        resp.raise_for_status()
        text = resp.json()["content"][0]["text"]
        text = re.sub(r"```json\s*", "", text)
        text = re.sub(r"```\s*", "", text)
        return json.loads(text)

    def _ollama_vision(self, prompt: str, b64_image: str) -> Optional[dict]:
        import requests
        url = f"{self.ollama_url}/api/generate"
        resp = requests.post(
            url,
            json={
                "model": self.vision_model,
                "prompt": prompt,
                "images": [b64_image],
                "stream": False,
            },
            timeout=60,
        )
        resp.raise_for_status()
        text = resp.json()["response"]
        text = re.sub(r"```json\s*", "", text)
        text = re.sub(r"```\s*", "", text)
        return json.loads(text)

    @staticmethod
    def _parse_solution(text: str) -> Solution:
        """Parse the structured solution response."""
        cause = ""
        fix = ""
        alternatives = []

        for line in text.split("\n"):
            line = line.strip()
            if line.upper().startswith("CAUSE:"):
                cause = line[6:].strip()
            elif line.upper().startswith("FIX:"):
                fix = line[4:].strip()
            elif line.upper().startswith("ALT:"):
                alternatives.append(line[4:].strip())

        # Fallback: if structured parsing fails, use full text
        if not cause and not fix:
            cause = text[:200]
            fix = text

        return Solution(cause=cause, fix=fix, alternatives=alternatives)


# ---------------------------------------------------------------------------
# Solution Delivery
# ---------------------------------------------------------------------------

class SolutionDelivery:
    """Delivers solutions to the user via configured method."""

    def __init__(self, config: dict):
        self.method = config["delivery"]["method"]
        self.webhook_url = config["delivery"].get("webhook_url")
        self.log_file = config["delivery"].get("log_file", "./error_solutions.log")
        self.sound = config["delivery"].get("sound", True)

    def deliver(self, error: DetectedError, solution: Solution):
        """Deliver the solution to the user."""
        if self.method == "notification":
            self._notify_desktop(error, solution)
        elif self.method == "terminal":
            self._notify_terminal(error, solution)
        elif self.method == "file":
            self._notify_file(error, solution)
        elif self.method == "webhook":
            self._notify_webhook(error, solution)

        # Always log to file as backup
        if self.method != "file":
            self._notify_file(error, solution)

    def _notify_desktop(self, error: DetectedError, solution: Solution):
        """Send desktop notification."""
        try:
            from plyer import notification

            severity_icon = {"critical": "\u274c", "warning": "\u26a0\ufe0f", "info": "\u2139\ufe0f"}.get(
                error.severity, "\u2139\ufe0f"
            )

            title = f"{severity_icon} {error.error_type}"
            message = f"{error.error_message[:100]}\n\nFix: {solution.fix[:150]}"

            notification.notify(
                title=title,
                message=message,
                timeout=10,
                app_name="Screen Error Solver",
            )
        except ImportError:
            logging.warning("plyer not installed, falling back to terminal")
            self._notify_terminal(error, solution)
        except Exception as e:
            logging.error(f"Desktop notification failed: {e}")
            self._notify_terminal(error, solution)

    def _notify_terminal(self, error: DetectedError, solution: Solution):
        """Print formatted error + solution to terminal."""
        severity_color = {
            "critical": "\033[91m",  # Red
            "warning": "\033[93m",   # Yellow
            "info": "\033[94m",      # Blue
        }.get(error.severity, "\033[0m")
        reset = "\033[0m"

        width = 60
        print()
        print(f"{severity_color}{'=' * width}{reset}")
        print(f"{severity_color}  ERROR DETECTED: {error.error_type}{reset}")
        print(f"{severity_color}{'-' * width}{reset}")
        print(f"  Message: {error.error_message[:200]}")
        if error.file_location:
            print(f"  File: {error.file_location}")
        print(f"{severity_color}{'-' * width}{reset}")
        print(f"  \033[92mCause:\033[0m {solution.cause}")
        print(f"  \033[92mFix:\033[0m {solution.fix}")
        for i, alt in enumerate(solution.alternatives, 1):
            print(f"  \033[92mAlt {i}:\033[0m {alt}")
        print(f"{severity_color}{'=' * width}{reset}")
        print()

    def _notify_file(self, error: DetectedError, solution: Solution):
        """Append to log file."""
        entry = {
            "timestamp": error.timestamp,
            "error_type": error.error_type,
            "error_message": error.error_message,
            "severity": error.severity,
            "file_location": error.file_location,
            "cause": solution.cause,
            "fix": solution.fix,
            "alternatives": solution.alternatives,
        }
        try:
            with open(self.log_file, "a") as f:
                f.write(json.dumps(entry) + "\n")
        except Exception as e:
            logging.error(f"Failed to write log: {e}")

    def _notify_webhook(self, error: DetectedError, solution: Solution):
        """POST to webhook URL."""
        if not self.webhook_url:
            logging.warning("No webhook URL configured")
            return
        try:
            import requests
            payload = {
                "text": (
                    f"*Error detected*: `{error.error_type}`\n"
                    f"> {error.error_message[:200]}\n"
                    f"*Fix:* {solution.fix}\n"
                    f"*Cause:* {solution.cause}"
                )
            }
            requests.post(self.webhook_url, json=payload, timeout=10)
        except Exception as e:
            logging.error(f"Webhook delivery failed: {e}")


# ---------------------------------------------------------------------------
# Main Loop
# ---------------------------------------------------------------------------

class ScreenErrorSolver:
    """Main orchestrator — capture → detect → solve → deliver loop."""

    def __init__(self, config: dict):
        self.config = config
        self.capture = ScreenCapture(
            monitor=config["capture"]["monitor"],
            region=config["capture"].get("region"),
            downscale=config["capture"]["downscale_factor"],
        )
        self.detector = ErrorDetector(config)
        self.delivery = SolutionDelivery(config)
        self.interval = config["capture"]["interval_seconds"]
        self._running = False
        self._stats = {"frames": 0, "errors": 0, "solutions": 0}

    def run(self):
        """Start the main watching loop."""
        self._running = True
        logging.info(
            f"Screen Error Solver started | "
            f"mode={self.config['detection']['mode']} | "
            f"provider={self.config['api']['provider']} | "
            f"interval={self.interval}s"
        )
        print("\n\033[92m[Screen Error Solver] Watching your screen...\033[0m")
        print(f"  Mode: {self.config['detection']['mode']}")
        print(f"  Provider: {self.config['api']['provider']}")
        print(f"  Interval: {self.interval}s")
        print(f"  Delivery: {self.config['delivery']['method']}")
        print("  Press Ctrl+C to stop.\n")

        try:
            while self._running:
                self._tick()
                time.sleep(self.interval)
        except KeyboardInterrupt:
            print("\n\033[93m[Screen Error Solver] Stopped.\033[0m")
            print(f"  Stats: {self._stats['frames']} frames analyzed, "
                  f"{self._stats['errors']} errors detected, "
                  f"{self._stats['solutions']} solutions delivered")
        finally:
            self.capture.close()

    def _tick(self):
        """Single capture-detect-solve cycle."""
        self._stats["frames"] += 1

        try:
            # Capture
            img, changed = self.capture.capture()
            if not changed:
                return  # Screen hasn't changed, skip analysis

            b64, _ = self.capture.capture_as_base64()

            # Detect
            error = self.detector.detect(img, b64)
            if not error:
                return

            self._stats["errors"] += 1
            logging.info(f"Error detected: {error.error_type} - {error.error_message[:100]}")

            # Solve
            solution = self.detector._llm.generate_solution(error)
            self._stats["solutions"] += 1

            # Deliver
            self.delivery.deliver(error, solution)

        except Exception as e:
            logging.error(f"Tick failed: {e}")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def load_config(path: Optional[str] = None) -> dict:
    """Load config from YAML file or use defaults."""
    config = DEFAULT_CONFIG.copy()
    if path and Path(path).exists():
        try:
            import yaml
            with open(path) as f:
                user_config = yaml.safe_load(f)
            # Deep merge
            for section in user_config:
                if section in config and isinstance(config[section], dict):
                    config[section].update(user_config[section])
                else:
                    config[section] = user_config[section]
        except ImportError:
            logging.warning("PyYAML not installed. Using default config.")
        except Exception as e:
            logging.warning(f"Failed to load config: {e}. Using defaults.")
    return config


def main():
    parser = argparse.ArgumentParser(
        description="Screen Error Solver — watches your screen and fixes errors"
    )
    parser.add_argument("--config", "-c", help="Path to config.yaml")
    parser.add_argument("--provider", "-p", choices=["gemini", "openai", "anthropic", "ollama"],
                        help="LLM provider")
    parser.add_argument("--model", "-m", help="Model name for both vision and solution")
    parser.add_argument("--mode", choices=["vision", "ocr", "hybrid"],
                        help="Detection mode")
    parser.add_argument("--region", help="Screen region: x,y,width,height")
    parser.add_argument("--interval", "-i", type=float, help="Capture interval in seconds")
    parser.add_argument("--delivery", "-d", choices=["notification", "terminal", "file", "webhook"],
                        help="Solution delivery method")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose logging")

    args = parser.parse_args()

    # Setup logging
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s [%(levelname)s] %(message)s",
        datefmt="%H:%M:%S",
    )

    # Load config
    config = load_config(args.config)

    # Apply CLI overrides
    if args.provider:
        config["api"]["provider"] = args.provider
        # Set appropriate API key env var
        env_map = {
            "gemini": "GEMINI_API_KEY",
            "openai": "OPENAI_API_KEY",
            "anthropic": "ANTHROPIC_API_KEY",
            "ollama": "",
        }
        config["api"]["api_key_env"] = env_map.get(args.provider, "")

    if args.model:
        config["detection"]["vision_model"] = args.model
        config["solution"]["model"] = args.model

    if args.mode:
        config["detection"]["mode"] = args.mode

    if args.region:
        parts = [int(x) for x in args.region.split(",")]
        if len(parts) == 4:
            config["capture"]["region"] = {
                "left": parts[0], "top": parts[1],
                "width": parts[2], "height": parts[3],
            }

    if args.interval:
        config["capture"]["interval_seconds"] = args.interval

    if args.delivery:
        config["delivery"]["method"] = args.delivery

    # Validate API key
    provider = config["api"]["provider"]
    if provider != "ollama":
        key_env = config["api"]["api_key_env"]
        if not os.environ.get(key_env):
            print(f"\033[91mError: {key_env} environment variable not set.\033[0m")
            print(f"Set it with: export {key_env}=your-api-key")
            sys.exit(1)

    # Run
    solver = ScreenErrorSolver(config)
    solver.run()


if __name__ == "__main__":
    main()
