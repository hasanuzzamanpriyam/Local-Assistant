import pyautogui


def click(x: int, y: int) -> None:
    pyautogui.click(x, y)


def double_click(x: int, y: int) -> None:
    pyautogui.doubleClick(x, y)


def right_click(x: int, y: int) -> None:
    pyautogui.rightClick(x, y)


def type_text(text: str) -> None:
    pyautogui.write(text)


def press(key: str) -> None:
    pyautogui.press(key)


def hotkey(*keys: str) -> None:
    pyautogui.hotkey(*keys)


def scroll(x: int, y: int, clicks: int) -> None:
    pyautogui.scroll(clicks, x=x, y=y)


def drag(x1: int, y1: int, x2: int, y2: int) -> None:
    pyautogui.dragTo(x2, y2, startX=x1, startY=y1)


def move(x: int, y: int) -> None:
    pyautogui.moveTo(x, y)


def get_cursor_pos() -> dict:
    x, y = pyautogui.position()
    return {"x": x, "y": y}
