import mit_tools
from PIL import Image
import numpy as np

png_pil_img = Image.open("temp/file.jpg")
if png_pil_img.mode != "RGB":
    png_pil_img = png_pil_img.convert("RGB")

img = np.asarray(png_pil_img)

# style, vertical
pdf = mit_tools.PangoRenderer("", False)
pdf.set_background(img)
# text, (x, y, width, height), font_size, (horizontal_alignment, vertical_alignment), (foreground_color, background_color), bg
pdf.add_text("Hello world", (0.0, 0.0, 100.0, 100.0), 12, ("center", "center"), (0xFF5733, 0xFF5733), None)
# filename, output format, add extension
pdf.save("hello.pdf", "pdf false", False)
pdf.save("hello.png", "png true", False)
pdf.save("hello.svg", "svg", False)