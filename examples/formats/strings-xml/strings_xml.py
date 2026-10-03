#!/usr/bin/env python3
"""Minimal Android strings.xml format plugin for intl-ai (protocol v1).

One request line on stdin, one response line on stdout:

  read : {"v":1,"op":"read","format":"xml","path":p,"content":"..."}
         -> {"v":1,"data":{"key": "text", ...}}
  write: {"v":1,"op":"write","format":"xml","path":p,"data":{...}}
         -> {"v":1,"content":"<xml text>"}
  errors: {"v":1,"error":"code[:detail]"} with exit 0; a nonzero exit is
         a plugin crash, not a format error.

Corpus model: a nested key -> string map. strings.xml is flat, so write
flattens nested objects into dotted names ("nav.home") and read keeps
names verbatim; intl-ai's own flatten/set_nested bridges the two shapes.
Only <string> elements round-trip; <plurals>/<string-array> are skipped
on read (this is a reference plugin, not a full Android adapter).
"""

import json
import sys
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape


class FormatError(Exception):
    pass


def flatten(obj, prefix=""):
    out = {}
    for key, value in obj.items():
        name = f"{prefix}.{key}" if prefix else key
        if isinstance(value, dict):
            out.update(flatten(value, name))
        elif isinstance(value, str):
            out[name] = value
        else:
            raise FormatError(f"write:leaf '{name}' is not a string")
    return out


def read(content):
    try:
        root = ET.fromstring(content)
    except ET.ParseError as e:
        raise FormatError(f"parse:{e}")
    if root.tag != "resources":
        raise FormatError("parse:root element must be <resources>")
    data = {}
    for child in root:
        if child.tag != "string":
            continue
        name = child.get("name")
        if not name:
            raise FormatError("parse:<string> missing name attribute")
        data[name] = child.text or ""
    return data


_ATTR_ENTITIES = {'"': "&quot;"}


def write(data):
    if not isinstance(data, dict):
        raise FormatError("write:data must be an object")
    lines = ['<?xml version="1.0" encoding="utf-8"?>', "<resources>"]
    for name, value in sorted(flatten(data).items()):
        attr = escape(name, _ATTR_ENTITIES)
        lines.append(f'    <string name="{attr}">{escape(value)}</string>')
    lines.append("</resources>")
    return "\n".join(lines) + "\n"


def respond(**fields):
    sys.stdout.write(json.dumps({"v": 1, **fields}) + "\n")
    sys.stdout.flush()


def main():
    line = sys.stdin.readline()
    try:
        req = json.loads(line)
    except json.JSONDecodeError as e:
        return respond(error=f"protocol:bad request json ({e})")
    if req.get("v") != 1:
        return respond(error=f"protocol:unsupported version {req.get('v')}")
    op = req.get("op")
    try:
        if op == "read":
            return respond(data=read(req.get("content", "")))
        if op == "write":
            return respond(content=write(req.get("data")))
        return respond(error=f"protocol:unknown op {op!r}")
    except FormatError as e:
        return respond(error=str(e))


if __name__ == "__main__":
    main()
