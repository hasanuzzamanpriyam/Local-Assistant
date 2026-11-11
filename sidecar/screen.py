import base64
import io
from mss import mss


def list_monitors() -> list[dict]:
    with mss() as sct:
        return [
            {"id": i, "x": m["left"], "y": m["top"], "width": m["width"], "height": m["height"]}
            for i, m in enumerate(sct.monitors[1:], start=1)
        ]


def capture() -> str:
    with mss() as sct:
        monitor = sct.monitors[0]
        img = sct.grab(monitor)
        buf = io.BytesIO()
        from PIL import Image
        Image.frombytes("RGB", img.size, img.rgb).save(buf, format="PNG")
        return base64.b64encode(buf.getvalue()).decode("ascii")


def capture_region(x: int, y: int, w: int, h: int) -> str:
    with mss() as sct:
        region = {"top": y, "left": x, "width": w, "height": h}
        img = sct.grab(region)
        buf = io.BytesIO()
        from PIL import Image
        Image.frombytes("RGB", img.size, img.rgb).save(buf, format="PNG")
        return base64.b64encode(buf.getvalue()).decode("ascii")
