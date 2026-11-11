import base64
import io
from PIL import Image

try:
    import pytesseract
    TESSERACT_AVAILABLE = True
except Exception:
    TESSERACT_AVAILABLE = False


def _decode_image(image_b64: str) -> Image.Image:
    img_data = base64.b64decode(image_b64)
    return Image.open(io.BytesIO(img_data))


def extract_text(image_b64: str) -> list[dict]:
    if not TESSERACT_AVAILABLE:
        return []
    img = _decode_image(image_b64)
    data = pytesseract.image_to_data(img, output_type=pytesseract.Output.DICT)
    results = []
    for i in range(len(data["text"])):
        text = data["text"][i].strip()
        if text:
            results.append({
                "text": text,
                "x": data["left"][i],
                "y": data["top"][i],
                "width": data["width"][i],
                "height": data["height"][i],
                "confidence": data["conf"][i],
            })
    return results


def find_text(image_b64: str, query: str) -> list[dict]:
    if not TESSERACT_AVAILABLE:
        return []
    all_text = extract_text(image_b64)
    query_lower = query.lower()
    return [item for item in all_text if query_lower in item["text"].lower()]
