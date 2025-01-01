First of all, thank you for contributing to Blueprint.

If you learn something useful, please add it to this file.

# Run the test suite

```sh
python -m unittest
```

# Formatting

Blueprint uses [Black](https://github.com/psf/black) for code formatting.

# Build the docs

```sh
pip install -U sphinx furo

meson -Ddocs=true build
# or
meson --reconfigure -Ddocs=true build

ninja -C build docs/en

python -m http.server 2310 --bind 127.0.0.1 --directory build/docs/en/
xdg-open http://127.0.0.1:2310/
```
