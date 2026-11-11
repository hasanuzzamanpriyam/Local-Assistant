from actions import screenshot, wait

def test_screenshot_returns_base64():
    result = screenshot()
    assert isinstance(result, str)
    assert len(result) > 0

def test_wait_returns_none():
    assert wait(0.1) is None
