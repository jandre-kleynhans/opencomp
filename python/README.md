# OpenComp Python SDK (Phase 3)

The typed Python interface to OpenComp projects. Agents load, build, animate,
and render compositions without hand-editing TOML.

## Install (dev)

```bash
cd python
python -m venv .venv
.venv/bin/pip install -e .
```

## Use

```python
from opencomp import Project, Layer, Keyframe, AnimProp
from opencomp.engine import render_frame_bytes

p = Project(name="demo", width=640, height=480)
layer = Layer(name="square", kind="solid", color="#ff0000", size=[100, 100])
layer.animate(AnimProp.POSITION, [
    Keyframe(time=0, value=[100, 200]),
    Keyframe(time=60, value=[500, 200], easing="ease_in_out"),
])
p.add(layer)

png = render_frame_bytes(p, 30)
open("frame_0030.png", "wb").write(png)
```

The SDK serializes to the same TOML format the Rust core understands.