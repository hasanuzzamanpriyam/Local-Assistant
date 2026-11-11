import json
from typing import Any, Optional

MSG_BOOTING = "booting"
MSG_LOADING = "loading"
MSG_READY = "ready"
MSG_LOAD_ERROR = "load_error"
MSG_TASK_STARTED = "task_started"
MSG_STEP = "step"
MSG_TASK_COMPLETE = "task_complete"
MSG_TASK_STOPPED = "task_stopped"
MSG_PONG = "pong"
MSG_ERROR = "error"

CMD_RUN = "run"
CMD_STOP = "stop"
CMD_PING = "ping"
CMD_SHUTDOWN = "shutdown"


def encode(msg: dict[str, Any]) -> str:
    return json.dumps(msg, separators=(",", ":"))


def decode(line: str) -> Optional[dict[str, Any]]:
    line = line.strip()
    if not line:
        return None
    try:
        return json.loads(line)
    except json.JSONDecodeError:
        return None
