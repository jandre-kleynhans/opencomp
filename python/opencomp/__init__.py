"""opencomp — Python SDK (Phase 3).

A typed, agent-friendly wrapper around the OpenComp TOML project format.
The Rust core remains the renderer; this package is the agent surface.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import List, Optional, Union
import subprocess
import tempfile


# --------------------------------------------------------------------------
# Value types
# --------------------------------------------------------------------------

@dataclass(frozen=True)
class Color:
    r: int
    g: int
    b: int
    a: int = 255

    def __post_init__(self):
        for v in (self.r, self.g, self.b, self.a):
            if not 0 <= v <= 255:
                raise ValueError(f"color channel out of range: {v}")

    @classmethod
    def from_hex(cls, s: str) -> "Color":
        s = s.lstrip("#")
        if len(s) == 6:
            return cls(int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16))
        if len(s) == 8:
            return cls(int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16), int(s[6:8], 16))
        raise ValueError(f"invalid hex color: {s}")

    def to_hex(self) -> str:
        return f"#{self.r:02x}{self.g:02x}{self.b:02x}"


class AnimProp(Enum):
    POSITION = "position"
    SCALE = "scale"
    OPACITY = "opacity"
    ROTATION = "rotation"


class Easing(Enum):
    LINEAR = "linear"
    EASE_IN = "ease_in"
    EASE_OUT = "ease_out"
    EASE_IN_OUT = "ease_in_out"
    STEP = "step"


# --------------------------------------------------------------------------
# Animation
# --------------------------------------------------------------------------

@dataclass
class Keyframe:
    time: int  # frame number (or "MM:SS:FF" string)
    value: Union[float, List[float]]  # scalar or [x, y]
    easing: str = "linear"
    property: Optional[str] = None  # set by Layer.animate()

    def to_toml_entries(self) -> List[str]:
        if isinstance(self.value, (int, float)):
            v = f"value = {float(self.value):g}"
        else:
            v = f"value = [{float(self.value[0]):g}, {float(self.value[1]):g}]"
        return [
            "[[layer.keyframe]]",
            f'property = "{self.property}"',
            f"time = {self.time}",
            v,
            f'easing = "{self.easing}"',
        ]


@dataclass
class Expression:
    property: str
    expr: str


# --------------------------------------------------------------------------
# Transform
# --------------------------------------------------------------------------

@dataclass
class Transform:
    position: List[float] = field(default_factory=lambda: [0.0, 0.0])
    scale: List[float] = field(default_factory=lambda: [1.0, 1.0])
    opacity: float = 100.0
    rotation: float = 0.0


# --------------------------------------------------------------------------
# Layer
# --------------------------------------------------------------------------

@dataclass
class Layer:
    name: str
    kind: str = "solid"
    color: Optional[Color] = None
    size: List[float] = field(default_factory=lambda: [0.0, 0.0])
    transform: Transform = field(default_factory=Transform)
    keyframes: List[Keyframe] = field(default_factory=list)
    expressions: List[Expression] = field(default_factory=list)
    blend: str = "normal"

    def __post_init__(self):
        if isinstance(self.color, str):
            self.color = Color.from_hex(self.color)

    def animate(self, prop: AnimProp, keyframes: List[Keyframe]) -> None:
        for kf in keyframes:
            kf.property = prop.value  # type: ignore[attr-defined]
        self.keyframes.extend(keyframes)

    def expr(self, prop: AnimProp, expr: str) -> None:
        self.expressions.append(Expression(property=prop.value, expr=expr))


# --------------------------------------------------------------------------
# Project
# --------------------------------------------------------------------------

@dataclass
class Project:
    name: str
    width: int = 640
    height: int = 480
    fps: int = 24
    duration: int = 120
    bg_color: Color = field(default_factory=lambda: Color(10, 10, 10))
    layers: List[Layer] = field(default_factory=list)

    def __post_init__(self):
        if isinstance(self.bg_color, str):
            self.bg_color = Color.from_hex(self.bg_color)

    def add(self, layer: Layer) -> None:
        self.layers.append(layer)

    # ---- serialization --------------------------------------------------

    def to_toml(self) -> str:
        lines = ["[project]",
                 f'name = "{self.name}"',
                 f"fps = {self.fps}",
                 f"width = {self.width}",
                 f"height = {self.height}",
                 f"duration = {self.duration}",
                 f'bg_color = "{self.bg_color.to_hex()}"',
                 ""]
        for layer in self.layers:
            lines.append("[[layer]]")
            lines.append(f'name = "{layer.name}"')
            lines.append(f'type = "{layer.kind}"')
            if layer.color:
                lines.append(f'color = "{layer.color.to_hex()}"')
            if layer.size != [0.0, 0.0]:
                lines.append(f"size = [{layer.size[0]:g}, {layer.size[1]:g}]")
            if layer.transform != Transform():
                lines.append("")
                lines.append("[layer.transform]")
                t = layer.transform
                lines.append(f"position = [{t.position[0]:g}, {t.position[1]:g}]")
                lines.append(f"scale = [{t.scale[0]:g}, {t.scale[1]:g}]")
                lines.append(f"opacity = {t.opacity:g}")
                lines.append(f"rotation = {t.rotation:g}")
            for kf in layer.keyframes:
                lines.append("")
                lines.append("[[layer.keyframe]]")
                lines.append(f'property = "{kf.property}"')
                lines.append(f"time = {kf.time}")
                if isinstance(kf.value, (int, float)):
                    lines.append(f"value = {float(kf.value):g}")
                else:
                    lines.append(f"value = [{float(kf.value[0]):g}, {float(kf.value[1]):g}]")
                lines.append(f'easing = "{kf.easing}"')
            for ex in layer.expressions:
                lines.append("")
                lines.append("[[layer.expression]]")
                lines.append(f'property = "{ex.property}"')
                lines.append(f'expr = "{ex.expr}"')
            lines.append("")
        return "\n".join(lines)

    @classmethod
    def from_toml(cls, src: str) -> "Project":
        """Parse a TOML project string back into a Project (round-trip).

        Uses Python 3.11 tomllib for parsing, then maps to dataclasses.
        """
        import tomllib
        data = tomllib.loads(src)
        info = data["project"]
        p = cls(name=info["name"],
                width=info.get("width", 640),
                height=info.get("height", 480),
                fps=info.get("fps", 24),
                duration=info.get("duration", 120),
                bg_color=Color.from_hex(info["bg_color"]))
        for raw in data.get("layer", []):
            layer = Layer(
                name=raw["name"],
                kind=raw.get("type", "solid"),
                color=Color.from_hex(raw["color"]) if raw.get("color") else None,
                size=[float(x) for x in raw.get("size", [0.0, 0.0])],
                blend=raw.get("blend", "normal"),
            )
            if "transform" in raw:
                t = raw["transform"]
                layer.transform.position = [float(x) for x in t.get("position", [0.0, 0.0])]
                layer.transform.scale = [float(x) for x in t.get("scale", [1.0, 1.0])]
                layer.transform.opacity = float(t.get("opacity", 100.0))
                layer.transform.rotation = float(t.get("rotation", 0.0))
            for kf in raw.get("keyframe", []):
                val = kf["value"]
                if isinstance(val, list):
                    value = [float(x) for x in val]
                else:
                    value = float(val)
                layer.keyframes.append(Keyframe(
                    time=kf["time"],
                    value=value,
                    easing=kf.get("easing", "linear"),
                ))
            for ex in raw.get("expression", []):
                layer.expressions.append(Expression(property=ex["property"], expr=ex["expr"]))
            p.layers.append(layer)
        return p

    def save(self, path: Path) -> None:
        Path(path).write_text(self.to_toml())

    @classmethod
    def load(cls, path: Path) -> "Project":
        return cls.from_toml(Path(path).read_text())

    # ---- equality for round-trip tests -----------------------------------

    def __eq__(self, other):
        if not isinstance(other, Project):
            return NotImplemented
        return (self.to_toml() == other.to_toml())