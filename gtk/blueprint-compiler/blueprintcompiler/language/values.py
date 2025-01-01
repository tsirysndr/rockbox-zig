# values.py
#
# Copyright 2022 James Westman <james@jwestman.net>
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

from blueprintcompiler.gir import ArrayType
from blueprintcompiler.lsp_utils import SemanticToken

from .common import *
from .contexts import ScopeCtx, ValueTypeCtx
from .gobject_object import Object
from .types import TypeName


class Translated(AstNode):
    grammar = AnyOf(
        ["_", "(", UseQuoted("string"), ")"],
        [
            "C_",
            "(",
            UseQuoted("context"),
            ",",
            UseQuoted("string"),
            ")",
        ],
    )

    @property
    def string(self) -> str:
        return self.tokens["string"]

    @property
    def translate_context(self) -> T.Optional[str]:
        return self.tokens["context"]

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.context[ValueTypeCtx].value_type
        if expected_type is not None and not expected_type.assignable_to(StringType()):
            raise CompileError(
                f"Cannot convert translated string to {expected_type.full_name}"
            )

    @docs()
    def ref_docs(self):
        return get_docs_section("Syntax Translated")


class TypeLiteral(AstNode):
    grammar = [
        "typeof",
        AnyOf(
            [
                "<",
                to_parse_node(TypeName).expected("type name"),
                Match(">").expected(),
            ],
            [
                UseExact("lparen", "("),
                to_parse_node(TypeName).expected("type name"),
                UseExact("rparen", ")").expected("')'"),
            ],
        ),
    ]

    @property
    def type(self):
        return gir.TypeType()

    @property
    def type_name(self) -> TypeName:
        return self.children[TypeName][0]

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.context[ValueTypeCtx].value_type
        if expected_type is not None and not isinstance(expected_type, gir.TypeType):
            raise CompileError(f"Cannot convert GType to {expected_type.full_name}")

    @validate("lparen", "rparen")
    def upgrade_to_angle_brackets(self):
        if self.tokens["lparen"]:
            raise UpgradeWarning(
                "Use angle bracket syntax introduced in blueprint 0.8.0",
                actions=[
                    CodeAction(
                        "Use <> instead of ()",
                        f"<{self.children[TypeName][0].as_string}>",
                    )
                ],
            )

    @docs()
    def ref_docs(self):
        return get_docs_section("Syntax TypeLiteral")


class QuotedLiteral(AstNode):
    grammar = UseQuoted("value")

    @property
    def value(self) -> str:
        return self.tokens["value"]

    @property
    def type(self):
        return gir.StringType()

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.context[ValueTypeCtx].value_type
        if (
            isinstance(expected_type, gir.IntType)
            or isinstance(expected_type, gir.UIntType)
            or isinstance(expected_type, gir.FloatType)
        ):
            raise CompileError(f"Cannot convert string to number")

        elif isinstance(expected_type, gir.StringType):
            pass

        elif (
            isinstance(expected_type, gir.Class)
            or isinstance(expected_type, gir.Interface)
            or isinstance(expected_type, gir.Boxed)
        ):
            parseable_types = [
                "Gdk.Paintable",
                "Gdk.Texture",
                "Gdk.Pixbuf",
                "Gio.File",
                "Gtk.ShortcutTrigger",
                "Gtk.ShortcutAction",
                "Gdk.RGBA",
                "Gdk.ContentFormats",
                "Gsk.Transform",
                "GLib.Variant",
            ]
            if expected_type.full_name not in parseable_types:
                hints = []
                if isinstance(expected_type, gir.TypeType):
                    hints.append(f"use the typeof operator: 'typeof({self.value})'")
                raise CompileError(
                    f"Cannot convert string to {expected_type.full_name}", hints=hints
                )

        elif expected_type is not None:
            raise CompileError(f"Cannot convert string to {expected_type.full_name}")


class NumberLiteral(AstNode):
    grammar = [
        Optional(AnyOf(UseExact("sign", "-"), UseExact("sign", "+"))),
        UseNumber("value"),
    ]

    @property
    def type(self) -> gir.GirType:
        if isinstance(self.value, int):
            return gir.IntType()
        else:
            return gir.FloatType()

    @property
    def value(self) -> T.Union[int, float]:
        if self.tokens["sign"] == "-":
            return -self.tokens["value"]
        else:
            return self.tokens["value"]

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.context[ValueTypeCtx].value_type
        if isinstance(expected_type, gir.IntType):
            if not isinstance(self.value, int):
                raise CompileError(
                    f"Cannot convert {self.group.tokens['value']} to integer"
                )

        elif isinstance(expected_type, gir.UIntType):
            if self.value < 0:
                raise CompileError(
                    f"Cannot convert -{self.group.tokens['value']} to unsigned integer"
                )

        elif not isinstance(expected_type, gir.FloatType) and expected_type is not None:
            raise CompileError(f"Cannot convert number to {expected_type.full_name}")


class Flag(AstNode):
    grammar = UseIdent("value")

    @property
    def name(self) -> str:
        return self.tokens["value"]

    @property
    def value(self) -> T.Optional[int]:
        type = self.context[ValueTypeCtx].value_type
        if not isinstance(type, Enumeration):
            return None
        elif member := type.members.get(self.name):
            return member.value
        else:
            return None

    def get_semantic_tokens(self) -> T.Iterator[SemanticToken]:
        yield SemanticToken(
            self.group.tokens["value"].start,
            self.group.tokens["value"].end,
            SemanticTokenType.EnumMember,
        )

    @docs()
    def docs(self):
        type = self.context[ValueTypeCtx].value_type
        if not isinstance(type, Enumeration):
            return
        if member := type.members.get(self.tokens["value"]):
            return member.doc

    @validate()
    def validate_for_type(self):
        expected_type = self.context[ValueTypeCtx].value_type
        if (
            isinstance(expected_type, gir.Bitfield)
            and self.tokens["value"] not in expected_type.members
        ):
            raise CompileError(
                f"{self.tokens['value']} is not a member of {expected_type.full_name}",
                did_you_mean=(self.tokens["value"], expected_type.members.keys()),
            )

    @validate()
    def unique(self):
        self.validate_unique_in_parent(
            f"Duplicate flag '{self.name}'", lambda x: x.name == self.name
        )


class Flags(AstNode):
    grammar = [Flag, "|", Flag, ZeroOrMore(["|", Flag])]

    @property
    def flags(self) -> T.List[Flag]:
        return self.children

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.context[ValueTypeCtx].value_type
        if expected_type is not None and not isinstance(expected_type, gir.Bitfield):
            raise CompileError(f"{expected_type.full_name} is not a bitfield type")

    @docs()
    def ref_docs(self):
        return get_docs_section("Syntax Flags")


class IdentLiteral(AstNode):
    grammar = UseIdent("value")

    @property
    def ident(self) -> str:
        return self.tokens["value"]

    @property
    def type(self) -> T.Optional[gir.GirType]:
        # If the expected type is known, then use that. Otherwise, guess.
        if expected_type := self.context[ValueTypeCtx].value_type:
            return expected_type
        elif self.ident in ["true", "false"]:
            return gir.BoolType()
        elif object := self.context[ScopeCtx].objects.get(self.ident):
            return object.gir_class
        elif self.root.is_legacy_template(self.ident):
            return self.root.template.class_name.gir_type
        else:
            return None

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.context[ValueTypeCtx].value_type
        if isinstance(expected_type, gir.BoolType):
            if self.ident not in ["true", "false"]:
                raise CompileError(f"Expected 'true' or 'false' for boolean value")

        elif isinstance(expected_type, gir.Enumeration):
            if self.ident not in expected_type.members:
                raise CompileError(
                    f"{self.ident} is not a member of {expected_type.full_name}",
                    did_you_mean=(self.ident, list(expected_type.members.keys())),
                )

        elif self.root.is_legacy_template(self.ident):
            raise UpgradeWarning(
                "Use 'template' instead of the class name (introduced in 0.8.0)",
                actions=[CodeAction("Use 'template'", "template")],
            )

        elif expected_type is not None or self.context[ValueTypeCtx].must_infer_type:
            object = self.context[ScopeCtx].objects.get(self.ident)
            if object is None:
                if self.ident == "null":
                    if not self.context[ValueTypeCtx].allow_null:
                        raise CompileError("null is not permitted here")
                else:
                    raise CompileError(
                        f"Could not find object with ID {self.ident}",
                        did_you_mean=(
                            self.ident,
                            self.context[ScopeCtx].objects.keys(),
                        ),
                    )
            elif (
                expected_type is not None
                and object.gir_class is not None
                and not object.gir_class.assignable_to(expected_type)
            ):
                raise CompileError(
                    f"Cannot assign {object.gir_class.full_name} to {expected_type.full_name}"
                )

    @docs()
    def docs(self) -> T.Optional[str]:
        expected_type = self.context[ValueTypeCtx].value_type
        if isinstance(expected_type, gir.BoolType):
            return None
        elif isinstance(expected_type, gir.Enumeration):
            if member := expected_type.members.get(self.ident):
                return member.doc
            else:
                return expected_type.doc
        elif self.ident == "null" and self.context[ValueTypeCtx].allow_null:
            return None
        elif object := self.context[ScopeCtx].objects.get(self.ident):
            return f"```\n{object.signature}\n```"
        elif self.root.is_legacy_template(self.ident):
            return f"```\n{self.root.template.signature}\n```"
        else:
            return None

    def get_semantic_tokens(self) -> T.Iterator[SemanticToken]:
        type = self.context[ValueTypeCtx].value_type
        if isinstance(type, gir.Enumeration):
            token = self.group.tokens["value"]
            yield SemanticToken(token.start, token.end, SemanticTokenType.EnumMember)

    def get_reference(self, _idx: int) -> T.Optional[LocationLink]:
        ref = self.context[ScopeCtx].objects.get(self.ident)
        if ref is None and self.root.is_legacy_template(self.ident):
            ref = self.root.template

        if ref:
            return LocationLink(self.range, ref.range, ref.ranges["id"])
        else:
            return None


class Literal(AstNode):
    grammar = AnyOf(
        TypeLiteral,
        QuotedLiteral,
        NumberLiteral,
        IdentLiteral,
    )

    @property
    def value(
        self,
    ) -> T.Union[TypeLiteral, QuotedLiteral, NumberLiteral, IdentLiteral]:
        return self.children[0]


class ObjectValue(AstNode):
    grammar = Object

    @property
    def object(self) -> Object:
        return self.children[Object][0]

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.context[ValueTypeCtx].value_type
        if (
            expected_type is not None
            and self.object.gir_class is not None
            and not self.object.gir_class.assignable_to(expected_type)
        ):
            raise CompileError(
                f"Cannot assign {self.object.gir_class.full_name} to {expected_type.full_name}"
            )


class Value(AstNode):
    grammar = AnyOf(Translated, Flags, Literal)

    @property
    def child(
        self,
    ) -> T.Union[Translated, Flags, Literal]:
        return self.children[0]


class ArrayValue(AstNode):
    grammar = ["[", Delimited(Value, ","), "]"]

    @validate()
    def validate_for_type(self) -> None:
        expected_type = self.gir_type
        if expected_type is not None and not isinstance(expected_type, gir.ArrayType):
            raise CompileError(f"Cannot assign array to {expected_type.full_name}")

        if expected_type is not None and not isinstance(
            expected_type.inner, StringType
        ):
            raise CompileError("Only string arrays are supported")

    @validate()
    def validate_invalid_newline(self) -> None:
        expected_type = self.gir_type
        if isinstance(expected_type, gir.ArrayType) and isinstance(
            expected_type.inner, StringType
        ):
            errors = []
            for value in self.values:
                if isinstance(value.child, Literal) and isinstance(
                    value.child.value, QuotedLiteral
                ):
                    quoted_literal = value.child.value
                    literal_value = quoted_literal.value
                    # literal_value can be None if there's an invalid escape sequence
                    if literal_value is not None and "\n" in literal_value:
                        errors.append(
                            CompileError(
                                "String literals inside arrays can't contain newlines",
                                range=quoted_literal.range,
                            )
                        )
            if len(errors) > 0:
                raise MultipleErrors(errors)

    @property
    def values(self) -> T.List[Value]:
        return self.children

    @property
    def gir_type(self):
        return self.parent.context[ValueTypeCtx].value_type

    @context(ValueTypeCtx)
    def child_value(self):
        if self.gir_type is None or not isinstance(self.gir_type, ArrayType):
            return ValueTypeCtx(None)
        else:
            return ValueTypeCtx(self.gir_type.inner)


class StringValue(AstNode):
    grammar = AnyOf(Translated, QuotedLiteral)

    @property
    def child(
        self,
    ) -> T.Union[Translated, QuotedLiteral]:
        return self.children[0]

    @property
    def string(self) -> str:
        if isinstance(self.child, Translated):
            return self.child.string
        else:
            return self.child.value

    @context(ValueTypeCtx)
    def value_type(self) -> ValueTypeCtx:
        return ValueTypeCtx(StringType())
