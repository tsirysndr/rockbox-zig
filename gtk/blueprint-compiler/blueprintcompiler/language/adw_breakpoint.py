# adw_breakpoint.py
#
# Copyright 2023 James Westman <james@jwestman.net>
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

from .common import *
from .contexts import ScopeCtx, ValueTypeCtx
from .gobject_object import Object, validate_parent_type
from .values import Value


class AdwBreakpointCondition(AstNode):
    grammar = [
        UseExact("kw", "condition"),
        "(",
        UseQuoted("condition"),
        Match(")").expected(),
    ]

    @property
    def condition(self) -> str:
        return self.tokens["condition"]

    @property
    def document_symbol(self) -> DocumentSymbol:
        return DocumentSymbol(
            "condition",
            SymbolKind.Property,
            self.range,
            self.group.tokens["kw"].range,
            self.condition,
        )

    @docs("kw")
    def keyword_docs(self):
        klass = self.root.gir.get_type("Breakpoint", "Adw")
        if klass is None:
            return None
        prop = klass.properties.get("condition")
        assert isinstance(prop, gir.Property)
        return prop.doc

    @validate()
    def unique(self):
        self.validate_unique_in_parent("Duplicate condition statement")


class AdwBreakpointSetter(AstNode):
    grammar = Statement(
        UseIdent("object"),
        Match(".").expected(),
        UseIdent("property"),
        Match(":").expected(),
        Value,
    )

    @property
    def object_id(self) -> str:
        return self.tokens["object"]

    @property
    def object(self) -> T.Optional[Object]:
        return self.context[ScopeCtx].objects.get(self.object_id)

    @property
    def property_name(self) -> T.Optional[str]:
        return self.tokens["property"]

    @property
    def value(self) -> Value:
        return self.children[Value][0]

    @property
    def gir_class(self) -> T.Optional[GirType]:
        if self.object is not None:
            return self.object.gir_class
        else:
            return None

    @property
    def gir_property(self) -> T.Optional[gir.Property]:
        if (
            self.gir_class is not None
            and not isinstance(self.gir_class, ExternType)
            and self.property_name is not None
        ):
            assert isinstance(self.gir_class, gir.Class) or isinstance(
                self.gir_class, gir.TemplateType
            )
            return self.gir_class.properties.get(self.property_name)
        else:
            return None

    @property
    def document_symbol(self) -> DocumentSymbol:
        return DocumentSymbol(
            f"{self.object_id}.{self.property_name}",
            SymbolKind.Property,
            self.range,
            self.group.tokens["object"].range,
            self.value.range.text,
        )

    def get_reference(self, idx: int) -> T.Optional[LocationLink]:
        if idx in self.group.tokens["object"].range:
            if self.object is not None:
                return LocationLink(
                    self.group.tokens["object"].range,
                    self.object.range,
                    self.object.ranges["id"],
                )

        return None

    @context(ValueTypeCtx)
    def value_type(self) -> ValueTypeCtx:
        if self.gir_property is not None:
            type = self.gir_property.type
        else:
            type = None

        return ValueTypeCtx(type, allow_null=True)

    @docs("object")
    def object_docs(self):
        if self.object is not None:
            return f"```\n{self.object.signature}\n```"
        else:
            return None

    @docs("property")
    def property_docs(self):
        if self.gir_property is not None:
            return self.gir_property.doc
        else:
            return None

    @validate("object")
    def object_exists(self):
        if self.object is None:
            raise CompileError(
                f"Could not find object with ID {self.object_id}",
                did_you_mean=(self.object_id, self.context[ScopeCtx].objects.keys()),
            )

    @validate("property")
    def property_exists(self):
        if self.gir_class is None or self.gir_class.incomplete:
            # Objects that we have no gir data on should not be validated
            # This happens for classes defined by the app itself
            return

        if self.gir_property is None and self.property_name is not None:
            raise CompileError(
                f"Class {self.gir_class.full_name} does not have a property called {self.property_name}",
                did_you_mean=(self.property_name, self.gir_class.properties.keys()),
            )

    @validate()
    def unique(self):
        self.validate_unique_in_parent(
            f"Duplicate setter for {self.object_id}.{self.property_name}",
            lambda x: x.object_id == self.object_id
            and x.property_name == self.property_name,
        )


class AdwBreakpointSetters(AstNode):
    grammar = [
        Keyword("setters"),
        Match("{").expected(),
        Until(AdwBreakpointSetter, "}"),
    ]

    @property
    def setters(self) -> T.List[AdwBreakpointSetter]:
        return self.children[AdwBreakpointSetter]

    @property
    def document_symbol(self) -> DocumentSymbol:
        return DocumentSymbol(
            "setters",
            SymbolKind.Struct,
            self.range,
            self.group.tokens["setters"].range,
        )

    @validate()
    def container_is_breakpoint(self):
        validate_parent_type(self, "Adw", "Breakpoint", "breakpoint setters")

    @validate()
    def unique(self):
        self.validate_unique_in_parent("Duplicate setters block")

    @docs("setters")
    def ref_docs(self):
        return get_docs_section("Syntax ExtAdwBreakpoint")


@decompiler("condition", cdata=True)
def decompile_condition(ctx: DecompileCtx, gir, cdata):
    ctx.print(f"condition({escape_quote(cdata)})")


@decompiler("setter", element=True)
def decompile_setter(ctx: DecompileCtx, gir, element):
    assert ctx.parent_node is not None
    # only run for the first setter
    for child in ctx.parent_node.children:
        if child.tag == "setter":
            if child != element:
                # already decompiled
                return
            else:
                break

    ctx.print("setters {")
    for child in ctx.parent_node.children:
        if child.tag == "setter":
            object_id = child["object"]
            property_name = child["property"]
            obj = ctx.find_object(object_id)
            if obj is not None:
                gir_class = ctx.type_by_cname(obj["class"])
            else:
                gir_class = None

            if object_id == ctx.template_class:
                object_id = "template"

            comments, string = ctx.decompile_value(
                child.cdata,
                gir_class,
                (child["translatable"], child["context"], child["comments"]),
            )
            ctx.print(f"{comments} {object_id}.{property_name}: {string};")
