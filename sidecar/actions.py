import time
from screen import capture
from input import click, type_text, press, hotkey, scroll


def screenshot() -> str:
    return capture()


def wait(seconds: float) -> None:
    time.sleep(seconds)


def click_element(identifier: str | tuple[int, int]) -> None:
    if isinstance(identifier, tuple):
        click(identifier[0], identifier[1])
    else:
        raise NotImplementedError("Text-based clicking requires OCR resolution in agent")


def type_into(text: str, target: str | None = None) -> None:
    if target:
        pass
    type_text(text)


def scroll_page(direction: str) -> None:
    x, y = 500, 500
    clicks = 5 if direction == "down" else -5
    scroll(x, y, clicks)


def hotkey_combo(keys: list[str]) -> None:
    hotkey(*keys)
