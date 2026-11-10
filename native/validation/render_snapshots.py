#!/usr/bin/env python3
"""Render deterministic business-page previews for offline visual review.

This is a design-aid renderer, not a GPU baseline. It consumes the same
framework-neutral snapshot records used by the native validators and applies
the Nana desktop layout tokens: 68px app bar, 236/72px navigation rail,
24px content padding, 16px cards and readable type hierarchy.
"""
from __future__ import annotations

import argparse
import os
import re
from pathlib import Path
from typing import Dict, List

from PIL import Image, ImageDraw, ImageFont

from nana_theme import palette

NAV_GROUPS = [
    # ASCII glyphs are intentional: they render consistently with the bundled
    # CJK font and mirror the stable icon labels used by the hosted tree.
    ("工作台", [("/", "项目", ">"), ("/projects", "资料库", "+"), ("/growth", "角色成长", "*"), ("/timeline", "事件时间线", "-")]),
    ("关系与分析", [("/audit", "规则审计", "!"), ("/graph", "关系图谱", "#"), ("/search", "分析与检索", "?")]),
    ("数据管线", [("/indexing", "索引", ">"), ("/backup", "备份", "<"), ("/simulation", "模拟", "o")]),
    ("系统", [("/agent", "智能体", "@"), ("/settings", "设置", "s"), ("/help", "帮助", "?"), ("/diagnostics", "诊断", "!"), ("/jobs", "任务", "="), ("/prompt-templates", "提示词模板", "p")]),
]
ROUTES = [item for _, group in NAV_GROUPS for item in group]
LAYOUT_HINTS = {
    "/": "当前项目 · 连接状态 · 最近活动", "/projects": "项目列表 · 详情检查器 · 新建/快照",
    "/growth": "特质表格 · 变化记录 · 预览", "/timeline": "按时间排序 · 事件详情 · 新建/更新",
    "/audit": "冲突事实 · 建议修复 · 运行审计", "/graph": "关系画布 · 中心实体 · 关系预设",
    "/search": "查询工具栏 · 结果列表 · 语义详情", "/indexing": "索引状态 · 分块进度 · 生成向量",
    "/backup": "导入导出 · 版本信息 · 快照检查", "/simulation": "场景编辑 · 阶段结果 · 风险提示",
    "/agent": "请求输入 · provider 状态 · 输出日志", "/settings": "安全配置 · 密钥状态 · 连接检查",
    "/help": "原生命令 · 参数说明 · 连接检查", "/diagnostics": "环境摘要 · 安全状态 · 生成诊断",
    "/jobs": "队列状态 · 运行日志 · 重试任务", "/prompt-templates": "模板编辑 · 变量预览 · 模板列表",
}
# Mirrors the route-specific NanaUI work surface in the hosted builder.  This
# is intentionally data-only so the offline renderer remains a design aid.
WORKSPACE_HINTS = {
    "/": ("工作台摘要", "最近活动 · 项目状态 · 待处理任务"),
    "/projects": ("项目检查器", "当前项目 · 项目列表 · 刷新项目"),
    "/growth": ("成长趋势", "特质变化 · 来源 · 时间线"),
    "/timeline": ("时间线筛选", "时间范围 · 事件详情 · 新建/更新"),
    "/audit": ("审计控制台", "冲突事实 · 修复建议 · 运行审计"),
    "/graph": ("关系画布", "中心实体 · 关系层级 · 关系预设"),
    "/search": ("检索工具栏", "关键词 · 结果列表 · 执行检索"),
    "/indexing": ("索引进度", "分块 · embedding · 重建索引"),
    "/backup": ("归档操作", "导入归档 · 导出项目 · 快照"),
    "/simulation": ("场景编辑器", "场景名称 · 触发条件 · 阶段目标"),
    "/agent": ("智能体会话", "请求 · Provider · 输出日志"),
    "/settings": ("提供方设置", "Provider · 密钥状态 · 测试连接"),
    "/help": ("命令浏览器", "命令筛选 · payload · 错误契约"),
    "/diagnostics": ("诊断检查", "环境 · 数据库 · Provider 状态"),
    "/jobs": ("任务队列", "当前任务 · 运行日志 · 刷新队列"),
    "/prompt-templates": ("模板编辑器", "模板选择 · 内容编辑 · 变量预览"),
}

def font(path: str, size: int):
    return ImageFont.truetype(path, size)


def parse(path: Path):
    page: Dict[str, str] = {}
    sections: List[Dict[str, str]] = []
    fields: List[Dict[str, str]] = []
    lists: List[Dict[str, str]] = []
    statuses: Dict[str, Dict[str, str]] = {}
    empties: List[Dict[str, str]] = []
    actions: List[Dict[str, str]] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        if not raw or raw.startswith("#"):
            if raw.startswith("# page|"):
                page = kv(raw[2:])
            continue
        kind, _, rest = raw.partition("|")
        data = kv(rest)
        if kind == "section" and (data.get("title") or data.get("label")):
            if not data.get("title"):
                data["title"] = data.get("label", "业务区块")
            sections.append(data)
        elif kind == "field" and data.get("state") and data.get("value"):
            fields.append(data)
        elif kind == "list" and data.get("items"):
            lists.append(data)
        elif kind == "status" and data.get("active") == "1":
            statuses[data.get("id", "")] = data
        elif kind == "empty" and data.get("visible") == "1":
            empties.append(data)
        elif kind == "action" and data.get("command"):
            actions.append(data)
    # Legacy contract rows are retained for validators; visual previews prefer
    # the richer PageModel section rows that carry descriptions.
    described = [section for section in sections if section.get("description")]
    if described:
        sections = described
    return page, sections, fields, lists, statuses, empties, actions


def kv(rest: str) -> Dict[str, str]:
    out: Dict[str, str] = {}
    for part in rest.split("|"):
        key, sep, value = part.partition("=")
        if sep:
            out[key] = value
    return out


def rounded(draw, box, fill, outline=None, radius=12, width=1):
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def truncate(value: str, limit: int = 34) -> str:
    return value if len(value) <= limit else value[: limit - 1] + "…"


def draw_workspace(draw, active, x, y, w, c, f_body, f_small):
    """Render the route's information architecture, not a generic placeholder."""
    title, detail = WORKSPACE_HINTS.get(active, ("业务工作区", "业务操作"))
    rounded(draw, (x, y, x + w, y + 140), c["surface2"], c["accent"], 12)
    draw.text((x + 16, y + 9), title, font=f_body, fill=c["ink"])
    # The mini composition mirrors the hosted builder's route-specific grouping.
    if active == "/":
        for i, label in enumerate(("项目状态", "最近活动", "待处理")):
            xx = x + 16 + i * ((w - 48) // 3)
            rounded(draw, (xx, y + 29, xx + (w - 64) // 3, y + 50), c["surface"], c["line"], 5)
            draw.text((xx + 8, y + 34), label, font=f_small, fill=c["muted"])
    elif active in ("/projects", "/help", "/jobs", "/prompt-templates"):
        split = x + w // 2
        draw.line((split, y + 26, split, y + 51), fill=c["line"], width=1)
        draw.text((x + 16, y + 34), "列表" if active != "/help" else "命令", font=f_small, fill=c["accent"])
        draw.text((split + 16, y + 34), "检查器" if active == "/projects" else ("日志" if active == "/jobs" else "编辑器"), font=f_small, fill=c["accent"])
    elif active in ("/growth", "/indexing"):
        for i in range(7):
            xx = x + 18 + i * ((w - 44) // 7)
            yy = y + 50 - (i % 4) * 6
            draw.line((xx, y + 49, xx, yy), fill=c["accent"], width=3)
        draw.text((x + w - 170, y + 34), "趋势 / 进度", font=f_small, fill=c["muted"])
    elif active in ("/timeline", "/search"):
        rounded(draw, (x + 16, y + 31, x + 180, y + 50), c["surface"], c["line"], 5)
        rounded(draw, (x + 190, y + 31, x + 290, y + 50), c["surface"], c["line"], 5)
        draw.line((x + 310, y + 40, x + w - 18, y + 40), fill=c["line"], width=2)
    elif active == "/audit":
        split = x + w // 2
        draw.line((split, y + 28, split, y + 51), fill=c["line"], width=1)
        draw.text((x + 16, y + 35), "冲突事实", font=f_small, fill=c["muted"])
        draw.text((split + 16, y + 35), "修复建议", font=f_small, fill=c["muted"])
    elif active == "/graph":
        cx, cy = x + w // 2, y + 42
        draw.line((cx - 62, cy, cx - 14, cy), fill=c["accent"], width=2)
        draw.line((cx + 14, cy, cx + 62, cy), fill=c["accent"], width=2)
        rounded(draw, (cx - 14, cy - 9, cx + 14, cy + 9), c["accent"], None, 5)
        rounded(draw, (cx - 78, cy - 9, cx - 62, cy + 9), c["surface"], c["line"], 4)
        rounded(draw, (cx + 62, cy - 9, cx + 78, cy + 9), c["surface"], c["line"], 4)
    elif active == "/backup":
        for i, label in enumerate(("导入归档", "导出项目")):
            xx = x + 16 + i * 110
            rounded(draw, (xx, y + 30, xx + 94, y + 51), c["surface"], c["line"], 5)
            draw.text((xx + 10, y + 35), label, font=f_small, fill=c["accent"])
    elif active == "/simulation":
        rounded(draw, (x + 16, y + 31, x + 170, y + 50), c["surface"], c["line"], 5)
        rounded(draw, (x + 180, y + 31, x + w - 18, y + 50), c["surface"], c["line"], 5)
    elif active == "/agent":
        split = x + w // 3
        draw.line((split, y + 28, split, y + 51), fill=c["line"], width=1)
        draw.text((x + 16, y + 35), "Provider", font=f_small, fill=c["muted"])
        draw.text((split + 16, y + 35), "请求 / 输出日志", font=f_small, fill=c["muted"])
    elif active == "/settings":
        for i in range(3):
            xx = x + 16 + i * ((w - 48) // 3)
            draw.line((xx, y + 48, xx + (w - 64) // 3, y + 48), fill=c["accent"], width=2)
        draw.text((x + 16, y + 34), "Provider", font=f_small, fill=c["muted"])
    elif active == "/diagnostics":
        for i, label in enumerate(("环境", "数据库", "Provider")):
            xx = x + 16 + i * ((w - 48) // 3)
            rounded(draw, (xx, y + 30, xx + (w - 64) // 3, y + 50), c["surface"], c["line"], 5)
            draw.text((xx + 8, y + 35), label, font=f_small, fill=c["muted"])
    else:
        draw.text((x + 16, y + 34), detail, font=f_small, fill=c["muted"])
    draw.line((x + 16, y + 66, x + w - 16, y + 66), fill=c["line"], width=1)
    draw.text((x + 16, y + 78), detail, font=f_small, fill=c["muted"])
    if active in ("/projects", "/help", "/jobs", "/prompt-templates"):
        for i in range(3):
            xx = x + 16 if i < 2 else x + w // 2 + 16
            yy = y + 101 + (i % 2) * 16
            draw.line((xx, yy, xx + (w // 2 - 34 if i == 2 else w // 2 - 50), yy), fill=c["line"], width=3)
    elif active in ("/growth", "/indexing"):
        draw.line((x + 18, y + 124, x + w - 18, y + 92), fill=c["accent"], width=2)
        draw.line((x + 18, y + 124, x + w - 18, y + 124), fill=c["line"], width=1)
    elif active in ("/timeline", "/search"):
        for i in range(4):
            xx = x + 18 + i * ((w - 52) // 4)
            draw.ellipse((xx, y + 110, xx + 7, y + 117), fill=c["accent"])
            draw.line((xx + 7, y + 113, xx + 75, y + 113), fill=c["line"], width=2)
    elif active == "/graph":
        draw.line((x + 60, y + 115, x + w - 60, y + 115), fill=c["accent"], width=2)
        draw.ellipse((x + w // 2 - 10, y + 105, x + w // 2 + 10, y + 125), fill=c["accent"])
    elif active == "/backup":
        draw.text((x + 16, y + 106), "归档版本 1 · SQLite · 可恢复", font=f_small, fill=c["muted"])
    elif active == "/simulation":
        draw.line((x + 16, y + 110, x + w - 16, y + 110), fill=c["line"], width=4)
        draw.line((x + 16, y + 126, x + w * 0.62, y + 126), fill=c["accent"], width=4)
    elif active == "/agent":
        draw.line((x + w // 3, y + 78, x + w // 3, y + 130), fill=c["line"], width=1)
        draw.text((x + w // 3 + 16, y + 106), "输出日志 / 结果", font=f_small, fill=c["muted"])
    elif active == "/settings":
        draw.text((x + 16, y + 106), "密钥安全存储 · 连接检查 · 模型配置", font=f_small, fill=c["muted"])
    elif active == "/diagnostics":
        draw.text((x + 16, y + 106), "环境检查通过 · 数据库检查通过 · 等待 provider", font=f_small, fill=c["muted"])


def render(snapshot: Path, output: Path, collapsed: bool, font_path: str):
    page, sections, fields, lists, statuses, empties, actions = parse(snapshot)
    c = palette(collapsed)
    im = Image.new("RGB", (1280, 800), c["bg"])
    d = ImageDraw.Draw(im)
    f_brand, f_title, f_body, f_meta, f_small = (font(font_path, n) for n in (20, 25, 14, 12, 11))
    top = 68
    nav_w = 72 if collapsed else 236
    content_pad = 24
    card_pad = 16
    nav_pad = 16
    gap = 16
    d.rectangle((0, 0, 1280, top), fill=c["nav"])
    d.rectangle((0, top, nav_w, 800), fill=c["nav"])
    d.text((content_pad, 16), "A" if collapsed else "Ameya", font=f_brand, fill=c["ink"])
    if not collapsed:
        d.text((120, 20), "当前项目 · 本地工作区", font=f_small, fill=c["nav_muted"])
        d.text((300, 22), "已保存", font=f_small, fill=c["nav_muted"])
    d.line((nav_w, top, nav_w, 800), fill=c["line"], width=1)

    active = page.get("route", "/")
    y = top + 14
    item_h = 38
    for group, routes in NAV_GROUPS:
        if not collapsed:
            d.text((24, y), group, font=f_small, fill=c["nav_muted"])
            y += 18
        for path, label, icon in routes:
            selected = path == active
            if selected:
                rounded(d, (nav_pad, y - 4, nav_w - nav_pad, y + item_h - 6), c["nav_active"], radius=9)
            if collapsed:
                d.text((28, y + 5), icon, font=f_body, fill=c["ink"] if selected else c["nav_muted"])
            else:
                d.text((24, y + 3), icon, font=f_small, fill=c["ink"] if selected else c["nav_muted"])
                d.text((48, y + 2), label, font=f_body, fill=c["ink"] if selected else c["nav_muted"])
                d.text((48, y + 19), path, font=f_small, fill=c["accent"] if selected else c["faint"])
            y += item_h
        y += 4

    content_x = nav_w + content_pad
    content_w = 1280 - content_x - content_pad
    d.text((content_x, 91), page.get("title", "页面"), font=f_title, fill=c["ink"])
    d.text((content_x, 124), page.get("route", "/"), font=f_small, fill=c["muted"])
    # Header action strip gives the business action contract a proper visual home.
    button_x = content_x + content_w
    for action in reversed(actions[:1]):
        label = truncate(action.get("label", action.get("command", "操作")), 12)
        w = max(88, d.textbbox((0, 0), label, font=f_body)[2] + 28)
        button_x -= w + 10
        rounded(d, (button_x, 94, button_x + w, 128), c["accent"], radius=8)
        tw = d.textbbox((0, 0), label, font=f_body)[2]
        d.text((button_x + (w - tw) / 2, 102), label, font=f_body, fill=c["on_accent"])

    # Summary card.
    summary = LAYOUT_HINTS.get(active, "业务数据 · 状态 · 可执行操作")
    rounded(d, (content_x, 156, content_x + content_w, 206), c["surface"], c["line"], 12)
    d.text((content_x + card_pad, 173), summary, font=f_body, fill=c["ink"])
    d.text((content_x + card_pad, 192), "数据来源：NativeCommandHost · 本地模式 · 已保存", font=f_small, fill=c["muted"])

    # Mirror the hosted route-specific information architecture in the design aid.
    workspace_y = 222
    draw_workspace(d, active, content_x, workspace_y, content_w, c, f_body, f_small)

    # Draw up to three model sections in a responsive two-column card grid.
    card_y = 378
    card_gap = gap
    field_value_offset = 112
    badge_inset = 8
    single_section = len(sections) <= 1
    card_w = content_w if single_section else (content_w - card_gap) // 2
    field_by_state: Dict[str, List[Dict[str, str]]] = {}
    for field in fields:
        field_by_state.setdefault(field.get("state", ""), []).append(field)
    list_by_section: Dict[str, List[Dict[str, str]]] = {}
    for item in lists:
        list_id = item.get("id", "")
        section_key = list_id[5:].rsplit("-", 1)[0] if list_id.startswith("list-") else list_id
        list_by_section.setdefault(section_key, []).append(item)
    section_h = 150 if single_section else 110
    section_row_step = section_h + gap
    # Keep the design-aid viewport balanced after the route workspace card;
    # additional sections remain available in the native snapshot contract.
    for index, section in enumerate(sections[:2]):
        x = content_x if single_section else content_x + (index % 2) * (card_w + card_gap)
        y0 = card_y if single_section else card_y + (index // 2) * section_row_step
        h = section_h
        rounded(d, (x, y0, x + card_w, y0 + h), c["surface"], c["line"], 12)
        d.text((x + card_pad, y0 + 14), truncate(section.get("title", section.get("label", "业务区块")), 26), font=f_body, fill=c["ink"])
        d.text((x + card_pad, y0 + 37), truncate(section.get("description", "业务内容"), 42), font=f_small, fill=c["muted"])
        sid = section.get("id", "")
        status = statuses.get("status-" + sid, {})
        badge = status.get("label", status.get("value", "可用"))
        badge_w = d.textbbox((0, 0), badge, font=f_small)[2] + 18
        badge_left = x + card_w - badge_w - card_pad
        rounded(d, (badge_left, y0 + 13, x + card_w - card_pad, y0 + 34), c["surface2"], c["line"], 8)
        d.text((badge_left + badge_inset, y0 + 17), badge, font=f_small, fill=c["ok"])
        yy = y0 + 66
        section_fields = field_by_state.get("status-" + sid, [])
        section_lists = list_by_section.get(sid, [])
        for field in section_fields[:2 if section_lists else 4]:
            label = truncate(field.get("label", "字段"), 14)
            value = truncate(field.get("value", "—"), 22)
            d.text((x + card_pad, yy), label, font=f_small, fill=c["muted"])
            d.text((x + card_pad + field_value_offset, yy), value, font=f_small, fill=c["ink"])
            yy += 19
        for current_list in section_lists[:1]:
            d.text((x + card_pad, yy + 1), truncate(current_list.get("label", "列表"), 16), font=f_small, fill=c["accent"])
            empty_id = current_list.get("empty", "")
            empty = next((item for item in empties if item.get("id") == empty_id), None)
            if empty:
                d.text((x + card_pad, yy + 18), truncate("空态：" + empty.get("text", "暂无数据"), 28), font=f_small, fill=c["muted"])
                yy += 17
            columns = [part.split(":", 2)[1] for part in current_list.get("columns", "").split(",") if ":" in part]
            if columns:
                col_w = max(48, (card_w - card_pad * 2) // min(3, len(columns)))
                yy += 18
                d.line((x + card_pad, yy + 12, x + card_w - card_pad, yy + 12), fill=c["line"], width=1)
                for col, label in enumerate(columns[:3]):
                    d.text((x + card_pad + col * col_w, yy), truncate(label, 10), font=f_small, fill=c["muted"])
                yy += 18
                for row_index, value in enumerate(("sample-1", "sample-2")):
                    d.text((x + card_pad, yy), value, font=f_small, fill=c["ink"])
                    if row_index == 0:
                        d.line((x + card_pad, yy + 14, x + card_w - card_pad, yy + 14), fill=c["line"], width=1)
                    yy += 16
        if not section_fields and not section_lists:
            d.text((x + card_pad, yy), "暂无数据 · 可通过快捷操作加载", font=f_small, fill=c["muted"])

    rows = 1 if single_section else (min(2, len(sections)) + 1) // 2
    action_y = card_y + rows * section_row_step
    action_h = 92
    rounded(d, (content_x, action_y, content_x + content_w, action_y + action_h), c["surface"], c["line"], 12)
    d.text((content_x + card_pad, action_y + 14), "快捷操作", font=f_body, fill=c["ink"])
    d.text((content_x + card_pad, action_y + 39), "通过原生命令边界执行，不在页面中暴露敏感凭据。", font=f_small, fill=c["muted"])
    action_x = content_x + card_pad
    action_button_y = action_y + 56
    action_button_h = 24
    for action in actions[1:4]:
        label = truncate(action.get("label", action.get("command", "操作")), 12)
        w = max(92, d.textbbox((0, 0), label, font=f_small)[2] + 24)
        rounded(d, (action_x, action_button_y, action_x + w, action_button_y + action_button_h), c["surface2"], c["line"], 6)
        d.text((action_x + 12, action_button_y + 4), label, font=f_small, fill=c["accent"])
        action_x += w + 8

    footer_y = action_y + action_h + gap
    rounded(d, (content_x, footer_y, content_x + content_w, footer_y + 72), c["surface"], c["line"], 12)
    d.text((content_x + card_pad, footer_y + 14), "连接状态", font=f_body, fill=c["ink"])
    d.text((content_x + card_pad, footer_y + 40), "● 本地数据库 · NativeCommandHost 已就绪", font=f_small, fill=c["ok"])
    footer_note = "离屏预览 · 可交互"
    footer_note_w = d.textbbox((0, 0), footer_note, font=f_small)[2]
    d.text((content_x + content_w - card_pad - footer_note_w, footer_y + 40), footer_note, font=f_small, fill=c["muted"])
    output.parent.mkdir(parents=True, exist_ok=True)
    im.save(output, "PNG", optimize=False)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", required=True)
    ap.add_argument("--output", required=True)
    ap.add_argument("--collapsed", action="store_true")
    ap.add_argument("--font", default="/c/Windows/Fonts/msyh.ttc")
    args = ap.parse_args()
    render(Path(args.input), Path(args.output), args.collapsed, args.font)


if __name__ == "__main__":
    main()
