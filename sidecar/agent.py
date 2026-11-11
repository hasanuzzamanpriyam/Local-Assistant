import time
from screen import capture
from ocr import extract_text
from actions import screenshot as action_screenshot, wait, click_element, type_into, scroll_page, hotkey_combo
from protocol import MSG_STEP, MSG_TASK_COMPLETE, MSG_ERROR


class AgentLoop:
    def __init__(self, emit=None, max_steps=10):
        self.emit = emit
        self.max_steps = max_steps
        self.llm_url = None

    def run(self, task: str, max_steps: int, llm_url: str) -> dict:
        self.llm_url = llm_url
        self.max_steps = max_steps
        start = time.time()
        steps = 0

        if self.emit:
            self.emit({"event": "task_started", "task": task, "max_steps": max_steps})

        history = []
        for step in range(1, max_steps + 1):
            steps = step

            # Observe
            img_b64 = capture()
            text_items = extract_text(img_b64)
            ocr_text = "\n".join(item["text"] for item in text_items)

            # Plan (call LLM)
            action = self._plan(task, img_b64, ocr_text, history)

            if self.emit:
                self.emit({
                    "event": MSG_STEP,
                    "step": step,
                    "max_steps": max_steps,
                    "thought": action.get("thought", ""),
                    "action": action.get("action", ""),
                    "screenshot_base64": img_b64,
                })

            history.append(action)

            # Act
            if action.get("action") == "done":
                duration_ms = int((time.time() - start) * 1000)
                if self.emit:
                    self.emit({
                        "event": MSG_TASK_COMPLETE,
                        "success": True,
                        "result": action.get("result", "Task complete"),
                        "steps": steps,
                        "duration_ms": duration_ms,
                    })
                return {"success": True, "steps": steps, "duration_ms": duration_ms}

            self._execute_action(action)

        # Max steps reached
        duration_ms = int((time.time() - start) * 1000)
        if self.emit:
            self.emit({
                "event": MSG_TASK_COMPLETE,
                "success": False,
                "result": f"Max steps ({max_steps}) reached",
                "steps": steps,
                "duration_ms": duration_ms,
            })
        return {"success": False, "steps": steps, "duration_ms": duration_ms}

    def _plan(self, task: str, img_b64: str, ocr_text: str, history: list) -> dict:
        import requests
        prompt = f"""Task: {task}
OCR Text: {ocr_text}
History: {history}
Decide the next action. Respond with JSON: {{"thought": "...", "action": "click|type|scroll|wait|hotkey|done", "target": "...", "text": "...", "result": "..."}}
"""
        try:
            resp = requests.post(
                f"{self.llm_url}/v1/chat/completions",
                json={"model": "default", "messages": [{"role": "user", "content": prompt}], "stream": False},
                timeout=30,
            )
            content = resp.json()["choices"][0]["message"]["content"]
            import json as _json
            return _json.loads(content)
        except Exception:
            return {"thought": "LLM unavailable", "action": "done", "result": "LLM call failed"}

    def _execute_action(self, action: dict) -> None:
        act = action.get("action", "")
        if act == "click":
            target = action.get("target", "")
            if target:
                click_element(target)
        elif act == "type":
            type_into(action.get("text", ""))
        elif act == "scroll":
            scroll_page(action.get("direction", "down"))
        elif act == "wait":
            wait(action.get("seconds", 1.0))
        elif act == "hotkey":
            hotkey_combo(action.get("keys", []))
