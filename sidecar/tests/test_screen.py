from screen import list_monitors, capture

def test_list_monitors_returns_list():
    monitors = list_monitors()
    assert isinstance(monitors, list)
    assert len(monitors) >= 1
    assert "width" in monitors[0]
    assert "height" in monitors[0]

def test_capture_returns_base64():
    img_b64 = capture()
    assert isinstance(img_b64, str)
    assert len(img_b64) > 0
    import base64
    base64.b64decode(img_b64)
