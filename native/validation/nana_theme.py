"""Single offline mirror of NanaUI's built-in semantic theme tokens.

The hosted NanaUI tree uses semantic roles from nana_ui_core::SemanticPalette;
this module is only the deterministic Pillow renderer's adapter to those same
roles. Page rendering code must not introduce its own colors.
"""

NANA_LIGHT = {
    "background": "#ffffff",
    "surface": "#f3f4f6",
    "subtle": "#f7f8fa",
    "text": "#1a1a1f",
    "muted": "#5a616e",
    "faint": "#9ca3af",
    "border": "#e3e5e8",
    "selected": "#e2e2e2",
    "accent": "#4991d7",
    "accent_strong": "#2c7ed6",
    "success": "#107e39",
    "warning": "#b8771c",
    "on_accent": "#ffffff",
}

NANA_DARK = {
    "background": "#181818",
    "surface": "#202020",
    "subtle": "#1c1c1c",
    "text": "#dddddd",
    "muted": "#a3a3a3",
    "faint": "#5a5a5a",
    "border": "#2a2a2a",
    "selected": "#353535",
    "accent": "#7bb9f0",
    "accent_strong": "#4991d7",
    "success": "#3fb950",
    "warning": "#d4a85b",
    "on_accent": "#0d1622",
}


def palette(collapsed: bool):
    """Return the Nana light/dark semantic role mapping for a preview state."""
    source = NANA_DARK if collapsed else NANA_LIGHT
    return {
        "bg": source["background"],
        "surface": source["surface"],
        "surface2": source["subtle"],
        "ink": source["text"],
        "muted": source["muted"],
        "faint": source["faint"],
        "line": source["border"],
        "nav": source["surface"],
        "nav_muted": source["muted"],
        "nav_active": source["selected"],
        "accent": source["accent"],
        "accent_strong": source["accent_strong"],
        "ok": source["success"],
        "warn": source["warning"],
        "on_accent": source["on_accent"],
    }
