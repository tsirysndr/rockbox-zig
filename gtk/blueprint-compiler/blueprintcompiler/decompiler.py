# decompiler.py
#
# Copyright 2021 James Westman <james@jwestman.net>
#
# This file is free software; you can redistribute it and/or modify it
# under the terms of the GNU Lesser General Public License as
# published by the Free Software Foundation; either version 3 of the
# License, or (at your option) any later version.
#
# This file is distributed in the hope that it will be useful, but
# WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
# Lesser General Public License for more details.
#
# You should have received a copy of the GNU Lesser General Public
# License along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# SPDX-License-Identifier: LGPL-3.0-or-later

import typing as T
from collections import defaultdict
from dataclasses import dataclass
from enum import Enum

from . import formatter
from .gir import *
from .utils import Colors, escape_quote
from .xml_reader import Element, parse, parse_string

__all__ = ["decompile"]


_DECOMPILERS: dict[str, list] = defaultdict(list)
_CLOSING = {
    "{": "}",
    "[": "]",
}
_NAMESPACES = [
    ("GLib", "2.0"),
    ("GObject", "2.0"),
    ("Gio", "2.0"),
    ("Adw", "1"),
]


class LineType(Enum):
    NONE = 1
    STMT = 2
    BLOCK_START = 3
    BLOCK_END = 4


class DecompileCtx:
    def __init__(self, parent_gir: T.Optional[GirContext] = None) -> None:
        self.sub_decompiler = parent_gir is not None
        self._result: str = ""
        self.gir = parent_gir or GirContext()
        self._blocks_need_end: T.List[str] = []
        self._last_line_type: LineType = LineType.NONE
        self._obj_type_stack: list[T.Optional[GirType]] = []
        self._node_stack: list[Element] = []

        self.gir.add_namespace(get_namespace("Gtk", "4.0"))

    @property
    def result(self) -> str:
        imports = ""

        if not self.sub_decompiler:
            import_lines = sorted(
                [
                    f"using {ns} {namespace.version};"
                    for ns, namespace in self.gir.namespaces.items()
                    if ns != "Gtk"
                ]
            )
            imports += "\n".join(["using Gtk 4.0;", *import_lines])

        return formatter.format(imports + self._result)

    def type_by_cname(self, cname: str) -> T.Optional[GirType]:
        if type := self.gir.get_type_by_cname(cname):
            return type

        for ns, version in _NAMESPACES:
            try:
                namespace = get_namespace(ns, version)
                if type := namespace.get_type_by_cname(cname):
                    self.gir.add_namespace(namespace)
                    return type
            except:
                pass

        return None

    def start_block(self) -> None:
        self._blocks_need_end.append("")
        self._obj_type_stack.append(None)

    def end_block(self) -> None:
        if close := self._blocks_need_end.pop():
            self.print(close)
        self._obj_type_stack.pop()

    @property
    def current_obj_type(self) -> T.Optional[GirType]:
        return next((x for x in reversed(self._obj_type_stack) if x is not None), None)

    def push_obj_type(self, type: T.Optional[GirType]) -> None:
        self._obj_type_stack[-1] = type

    @property
    def current_node(self) -> T.Optional[Element]:
        if len(self._node_stack) == 0:
            return None
        else:
            return self._node_stack[-1]

    @property
    def parent_node(self) -> T.Optional[Element]:
        if len(self._node_stack) < 2:
            return None
        else:
            return self._node_stack[-2]

    @property
    def root_node(self) -> T.Optional[Element]:
        if len(self._node_stack) == 0:
            return None
        else:
            return self._node_stack[0]

    @property
    def template_class(self) -> T.Optional[str]:
        assert self.root_node is not None
        for child in self.root_node.children:
            if child.tag == "template":
                return child["class"]

        return None

    def find_object(self, id: str) -> T.Optional[Element]:
        assert self.root_node is not None
        for child in self.root_node.children:
            if child.tag == "template" and child["class"] == id:
                return child

        def find_in_children(node: Element) -> T.Optional[Element]:
            if node.tag in ["object", "menu"] and node["id"] == id:
                return node
            else:
                for child in node.children:
                    if result := find_in_children(child):
                        return result
            return None

        return find_in_children(self.root_node)

    def end_block_with(self, text: str) -> None:
        self._blocks_need_end[-1] = text

    def print(self, line: str, newline: bool = True) -> None:
        self._result += line

        if line.endswith("{") or line.endswith("["):
            if len(self._blocks_need_end):
                self._blocks_need_end[-1] = _CLOSING[line[-1]]

    # Converts a value from an XML element to a blueprint string
    # based on the given type. Returns a tuple of translator comments
    # (if any) and the decompiled syntax.
    def decompile_value(
        self,
        value: str,
        type: T.Optional[GirType],
        translatable: T.Optional[T.Tuple[str, str, str]] = None,
    ) -> T.Tuple[str, str]:
        def get_enum_name(value):
            for member in type.members.values():
                if (
                    member.nick == value
                    or member.c_ident == value
                    or str(member.value) == value
                ):
                    return member.name
            return value.replace("-", "_")

        if translatable is not None and truthy(translatable[0]):
            return decompile_translatable(value, *translatable)
        elif type is None:
            return "", f"{escape_quote(value)}"
        elif type.assignable_to(FloatType()):
            return "", str(value)
        elif type.assignable_to(BoolType()):
            val = truthy(value)
            return "", ("true" if val else "false")
        elif type.assignable_to(ArrayType(StringType())):
            items = ", ".join([escape_quote(x) for x in value.split("\n")])
            return "", f"[{items}]"
        elif (
            type.assignable_to(self.gir.namespaces["Gtk"].lookup_type("Gdk.Pixbuf"))
            or type.assignable_to(self.gir.namespaces["Gtk"].lookup_type("Gdk.Texture"))
            or type.assignable_to(
                self.gir.namespaces["Gtk"].lookup_type("Gdk.Paintable")
            )
            or type.assignable_to(
                self.gir.namespaces["Gtk"].lookup_type("Gtk.ShortcutAction")
            )
            or type.assignable_to(
                self.gir.namespaces["Gtk"].lookup_type("Gtk.ShortcutTrigger")
            )
        ):
            return "", escape_quote(value)
        elif value == self.template_class:
            return "", "template"
        elif type.assignable_to(
            self.gir.namespaces["Gtk"].lookup_type("GObject.Object")
        ) or isinstance(type, Interface):
            return "", ("null" if value == "" else value)
        elif isinstance(type, Bitfield):
            flags = [get_enum_name(flag) for flag in value.split("|")]
            return "", " | ".join(flags)
        elif isinstance(type, Enumeration):
            return "", get_enum_name(value)
        elif isinstance(type, TypeType):
            if t := self.type_by_cname(value):
                return "", f"typeof<{full_name(t)}>"
            else:
                return "", f"typeof<${value}>"
        else:
            return "", escape_quote(value)


def decompile_element(
    ctx: DecompileCtx, gir: T.Optional[GirContext], xml: Element
) -> None:
    try:
        decompilers = [d for d in _DECOMPILERS[xml.tag] if d._filter(ctx)]
        if len(decompilers) == 0:
            raise UnsupportedError(f"unsupported XML tag: <{xml.tag}>")

        decompiler = decompilers[0]

        if decompiler._element:
            args = [ctx, gir, xml]
            kwargs: T.Dict[str, T.Optional[str]] = {}
        else:
            args = [ctx, gir]
            kwargs = {canon(name): value for name, value in xml.attrs.items()}
            if decompiler._cdata:
                if len(xml.children):
                    kwargs["cdata"] = None
                else:
                    kwargs["cdata"] = xml.cdata

        ctx._node_stack.append(xml)
        ctx.start_block()
        gir = decompiler(*args, **kwargs)

        if not decompiler._skip_children:
            for child in xml.children:
                decompile_element(ctx, gir, child)

        ctx.end_block()
        ctx._node_stack.pop()

    except UnsupportedError as e:
        raise e
    except TypeError as e:
        raise UnsupportedError(tag=xml.tag)


def decompile(data: str) -> str:
    ctx = DecompileCtx()

    xml = parse(data)
    decompile_element(ctx, None, xml)

    return ctx.result


def decompile_string(data: str) -> str:
    ctx = DecompileCtx()

    xml = parse_string(data)
    decompile_element(ctx, None, xml)

    return ctx.result


def canon(string: str) -> str:
    if string == "class":
        return "klass"
    else:
        return string.replace("-", "_").lower()


def truthy(string: str) -> bool:
    return string is not None and string.lower() in ["yes", "true", "t", "y", "1"]


def full_name(gir: GirType) -> str:
    return gir.name if gir.full_name.startswith("Gtk.") else gir.full_name


def lookup_by_cname(gir, cname: str) -> T.Optional[GirType]:
    if isinstance(gir, GirContext):
        return gir.get_type_by_cname(cname)
    else:
        return gir.get_containing(Repository).get_type_by_cname(cname)


def decompiler(
    tag,
    cdata=False,
    parent_type: T.Optional[str] = None,
    parent_tag: T.Optional[str] = None,
    skip_children=False,
    element=False,
):
    def decorator(func):
        func._cdata = cdata
        func._skip_children = skip_children
        func._element = element

        def filter(ctx):
            if parent_type is not None:
                if (
                    ctx.current_obj_type is None
                    or ctx.current_obj_type.full_name != parent_type
                ):
                    return False

            if parent_tag is not None:
                if not any(x.tag == parent_tag for x in ctx._node_stack):
                    return False

            return True

        func._filter = filter

        _DECOMPILERS[tag].append(func)
        return func

    return decorator


@decompiler("interface")
def decompile_interface(ctx, gir, domain=None):
    if domain is not None:
        ctx.print(f"translation-domain {escape_quote(domain)};")
    return gir


@decompiler("requires")
def decompile_requires(ctx, gir, lib=None, version=None):
    return gir


@decompiler("placeholder")
def decompile_placeholder(ctx, gir):
    pass


def decompile_translatable(
    string: str,
    translatable: T.Optional[str],
    context: T.Optional[str],
    comments: T.Optional[str],
) -> T.Tuple[str, str]:
    if translatable is not None and truthy(translatable):
        if comments is None:
            comments = ""
        else:
            comments = comments.replace("/*", " ").replace("*/", " ")
            comments = f"/* Translators: {comments} */"

        if context is not None:
            return comments, f"C_({escape_quote(context)}, {escape_quote(string)})"
        else:
            return comments, f"_({escape_quote(string)})"
    else:
        return "", f"{escape_quote(string)}"


@decompiler("property", cdata=True)
def decompile_property(
    ctx: DecompileCtx,
    gir,
    name,
    cdata,
    bind_source=None,
    bind_property=None,
    bind_flags=None,
    translatable="false",
    comments=None,
    context=None,
):
    name = name.replace("_", "-")
    if cdata is None:
        ctx.print(f"{name}: ")
        ctx.end_block_with(";")
    elif bind_source:
        flags = ""
        bind_flags = bind_flags or []
        if "sync-create" not in bind_flags:
            flags += " no-sync-create"
        if "invert-boolean" in bind_flags:
            flags += " inverted"
        if "bidirectional" in bind_flags:
            flags += " bidirectional"

        if bind_source == ctx.template_class:
            bind_source = "template"

        ctx.print(f"{name}: bind {bind_source}.{bind_property}{flags};")
    elif truthy(translatable):
        comments, translatable = decompile_translatable(
            cdata, translatable, context, comments
        )
        if comments is not None:
            ctx.print(comments)
        ctx.print(f"{name}: {translatable};")
    elif gir is None or gir.properties.get(name) is None:
        ctx.print(f"{name}: {escape_quote(cdata)};")
    elif (
        gir.assignable_to(ctx.gir.get_class("BuilderListItemFactory", "Gtk"))
        and name == "bytes"
    ):
        sub_ctx = DecompileCtx(ctx.gir)

        xml = parse_string(cdata)
        decompile_element(sub_ctx, None, xml)

        ctx.print(sub_ctx.result)
    else:
        _, string = ctx.decompile_value(cdata, gir.properties.get(name).type)
        ctx.print(f"{name}: {string};")
    return gir


@decompiler("attribute", cdata=True)
def decompile_attribute(
    ctx, gir, name, cdata, translatable="false", comments=None, context=None
):
    decompile_property(
        ctx,
        gir,
        name,
        cdata,
        translatable=translatable,
        comments=comments,
        context=context,
    )


@decompiler("attributes")
def decompile_attributes(ctx, gir):
    ctx.print("attributes {")


@dataclass
class UnsupportedError(Exception):
    message: str = "unsupported feature"
    tag: T.Optional[str] = None

    def print(self, filename: str):
        print(f"\n{Colors.RED}{Colors.BOLD}error: {self.message}{Colors.CLEAR}")
        print(f"in {Colors.UNDERLINE}{filename}{Colors.NO_UNDERLINE}")
        if self.tag:
            print(f"in tag {Colors.BLUE}{self.tag}{Colors.CLEAR}")
        print(
            f"""{Colors.FAINT}The compiler might support this feature, but the porting tool does not. You
probably need to port this file manually.{Colors.CLEAR}\n"""
        )
