import sys
import json
import threading
import os
from protocol import encode, decode, CMD_RUN, CMD_STOP, CMD_PING, CMD_SHUTDOWN, MSG_READY, MSG_BOOTING, MSG_LOAD_ERROR, MSG_ERROR
from agent import AgentLoop


class Runner:
    def __init__(self):
        self.tasks = {}
        self.running = False
        self.llm_url = "http://127.0.0.1:11434/v1"

    def start(self):
        self.running = True
        print(encode({"event": "booting", "pid": os.getpid()}), flush=True)
        print(encode({"event": "ready"}), flush=True)

        for line in sys.stdin:
            msg = decode(line)
            if msg is None:
                continue
            response = self.handle_command(msg)
            if response:
                print(encode(response), flush=True)
            if msg.get("cmd") == CMD_SHUTDOWN:
                break

    def handle_command(self, cmd: dict) -> dict | None:
        c = cmd.get("cmd")
        if c == CMD_PING:
            return {"cmd": "pong"}
        elif c == CMD_RUN:
            task_id = cmd.get("id")
            task = cmd.get("task")
            max_steps = cmd.get("max_steps", 10)
            agent = AgentLoop(emit=self._make_emit(task_id))
            thread = threading.Thread(target=agent.run, args=(task, max_steps, self.llm_url), daemon=True)
            self.tasks[task_id] = {"agent": agent, "thread": thread}
            thread.start()
            return {"event": "task_started", "id": task_id}
        elif c == CMD_STOP:
            task_id = cmd.get("id")
            if task_id in self.tasks:
                del self.tasks[task_id]
            return {"event": "task_stopped", "id": task_id}
        elif c == CMD_SHUTDOWN:
            self.running = False
            return {"event": "shutdown"}
        return None

    def _make_emit(self, task_id: str):
        def emit(event: dict):
            event["id"] = task_id
            print(encode(event), flush=True)
        return emit


if __name__ == "__main__":
    Runner().start()
