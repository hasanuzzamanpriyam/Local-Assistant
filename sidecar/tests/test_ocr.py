from ocr import extract_text, find_text

def test_extract_text_returns_list():
    import base64
    from PIL import Image
    import io
    img = Image.new("RGB", (100, 50), color="white")
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    b64 = base64.b64encode(buf.getvalue()).decode("ascii")
    result = extract_text(b64)
    assert isinstance(result, list)

def test_find_text_returns_list():
    import base64
    from PIL import Image
    import io
    img = Image.new("RGB", (100, 50), color="white")
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    b64 = base64.b64encode(buf.getvalue()).decode("ascii")
    result = find_text(b64, "hello")
    assert isinstance(result, list)
