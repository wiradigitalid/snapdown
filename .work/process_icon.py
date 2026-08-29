import os
from PIL import Image

src_path = r"C:\Users\kodes\Downloads\ChatGPT Image Aug 24, 2026, 10_12_43 PM.png"
icons_dir = r"apps\desktop\src-tauri\icons"
ui_assets_dir = r"apps\desktop\src\assets"

os.makedirs(icons_dir, exist_ok=True)
os.makedirs(ui_assets_dir, exist_ok=True)

img = Image.open(src_path).convert("RGBA")
print("Original size:", img.size)

# Save master PNG
img.save(os.path.join(icons_dir, "icon.png"), "PNG")
img.save(os.path.join(ui_assets_dir, "app-icon.png"), "PNG")

# 128x128
img_128 = img.resize((128, 128), Image.Resampling.LANCZOS)
img_128.save(os.path.join(icons_dir, "128x128.png"), "PNG")

# 32x32 for tray
img_32 = img.resize((32, 32), Image.Resampling.LANCZOS)
img_32.save(os.path.join(icons_dir, "32x32.png"), "PNG")

# Generate icon.ico
sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
img.save(os.path.join(icons_dir, "icon.ico"), format="ICO", sizes=sizes)
print("Icon processing successfully finished!")
