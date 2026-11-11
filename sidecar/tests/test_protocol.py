from protocol import encode, decode, MSG_TASK_STARTED, MSG_STEP, MSG_TASK_COMPLETE

def test_encode_decode_roundtrip():
    msg = {"event": "step", "id": "t1", "step": 1, "thought": "click button"}
    encoded = encode(msg)
    decoded = decode(encoded)
    assert decoded["event"] == "step"
    assert decoded["id"] == "t1"
    assert decoded["step"] == 1
    assert decoded["thought"] == "click button"

def test_decode_invalid_json_returns_none():
    assert decode("not json") is None

def test_decode_empty_line_returns_none():
    assert decode("") is None
