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
# text, (x, y, width, height), font_size, vertical_allignment, horizontal_alignment, font_color
pdf.add_text("Hello world", (0.0, 0.0, 100.0, 100.0), 12, "center", "center", 0xFF5733)
# filename, width, height, output format
pdf.save("hello.pdf", 6000, 6000, "pdf false")
pdf.save("hello.png", 6000, 6000, "png true")
pdf.save("hello.svg", 6000, 6000, "svg")