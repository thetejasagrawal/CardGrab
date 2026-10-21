# App icon exports

`cardgrabicon.png` is the transparent master artwork. The About screen uses a
copy at `public/cardgrab-icon.png`.

On macOS, with Pillow installed in your Python environment, regenerate the
desktop PNG, ICNS, and ICO assets with:

```sh
python3 scripts/generate-desktop-icons.py
```

Use this exporter for the final desktop assets. The default Tauri icon exporter
uses Lanczos resizing, which produces bright edge overshoot and faint detached
alpha rows along this icon's top and bottom. The desktop exporter uses Hamming
resampling in premultiplied-alpha space and checks the packaged ICNS images for
those bands. If you run `tauri icon` to update mobile assets, run this script
afterward to restore the desktop exports.
