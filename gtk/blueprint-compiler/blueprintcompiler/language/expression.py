# expressions.py
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


from ..decompiler import decompile_element
from .common import *
from .contexts import ScopeCtx, ValueTypeCtx
from .types import TypeName

expr = Sequence()


class ExprBase(AstNode):
    @context(ValueTypeCtx)
    def value_type(self) -> ValueTypeCtx:
        if rhs := self.rhs:
            return rhs.context[ValueTypeCtx]
        else:
            return self.parent.context[ValueTypeCtx]

    @property
    def type(self) -> T.Optional[GirType]:
        raise NotImplementedError()

    @property
    def type_complete(self) -> bool:
        return True

    @property
    def rhs(self) -> T.Optional["ExprBase"]:
        if isinstance(self.parent, Expression):
            children = list(self.parent.children)
            if children.index(self) + 1 < len(children):
                return children[children.index(self) + 1]
            else:
                return self.parent.rhs
        else:
            return None


class Expression(ExprBase):
    grammar = expr

    @property
    def last(self) -> ExprBase:
        return self.children[-1]

    @property
    def type(self) -> T.Optional[GirType]:
        return self.last.type

    @property
    def type_complete(self) -> bool:
        return self.last.type_complete


class InfixExpr(ExprBase):
    @property
    def lhs(self):
        children = list(self.parent_by_type(Expression).children)
        return children[children.index(self) - 1]


class LiteralExpr(ExprBase):
    grammar = LITERAL

    @property
    def is_object(self) -> bool:
        from .values import IdentLiteral

        return isinstance(self.literal.value, IdentLiteral) and (
            self.literal.value.ident in self.context[ScopeCtx].objects
            or self.root.is_legacy_template(self.literal.value.ident)
        )

    @property
    def literal(self):
        from .values import Literal

        return self.children[Literal][0]

    @property
    def type(self) -> T.Optional[GirType]:
        return self.literal.value.type

    @property
    def type_complete(self) -> bool:
        from .values import IdentLiteral

        if isinstance(self.literal.value, IdentLiteral):
            if object := self.context[ScopeCtx].objects.get(self.literal.value.ident):
                return not object.gir_class.incomplete
        return True


class LookupOp(InfixExpr):
    grammar = [".", UseIdent("property")]

    @context(ValueTypeCtx)
    def value_type(self) -> ValueTypeCtx:
        return ValueTypeCtx(None, must_infer_type=True)

    @property
    def property_name(self) -> str:
        return self.tokens["property"]

    @property
    def type(self) -> T.Optional[GirType]:
        if isinstance(self.lhs.type, gir.Class) or isinstance(
            self.lhs.type, gir.Interface
        ):
            if property := self.lhs.type.properties.get(self.property_name):
                return property.type

        return None

    @docs("property")
    def property_docs(self):
        if not (
            isinstance(self.lhs.type, gir.Class)
            or isinstance(self.lhs.type, gir.Interface)
        ):
            return None

        if property := self.lhs.type.properties.get(self.property_name):
            return property.doc

    @validate("property")
    def property_exists(self):
        if self.lhs.type is None:
            # Literal values throw their own errors if the type isn't known
            if isinstance(self.lhs, LiteralExpr):
                return

            raise CompileError(
                f"Could not determine the type of the preceding expression",
                hints=[
                    f"add a type cast so blueprint knows which type the property {self.property_name} belongs to"
                ],
            )

        if self.lhs.type.incomplete:
            return

        elif not isinstance(self.lhs.type, gir.Class) and not isinstance(
            self.lhs.type, gir.Interface
        ):
            raise CompileError(
                f"Type {self.lhs.type.full_name} does not have properties"
            )

        elif self.lhs.type.properties.get(self.property_name) is None:
            raise CompileError(
                f"{self.lhs.type.full_name} does not have a property called {self.property_name}",
                did_you_mean=(self.property_name, self.lhs.type.properties.keys()),
            )

    @validate("property")
    def property_deprecated(self):
        if self.lhs.type is None or not (
            isinstance(self.lhs.type, gir.Class)
            or isinstance(self.lhs.type, gir.Interface)
        ):
            return

        if property := self.lhs.type.properties.get(self.property_name):
            if property.deprecated:
                hints = []
                if property.deprecated_doc:
                    hints.append(property.deprecated_doc)
                raise DeprecatedWarning(
                    f"{property.signature} is deprecated",
                    hints=hints,
                )


class CastExpr(InfixExpr):
    grammar = [
        Keyword("as"),
        AnyOf(
            ["<", TypeName, Match(">").expected()],
            [
                UseExact("lparen", "("),
                TypeName,
                UseExact("rparen", ")").expected("')'"),
            ],
        ),
    ]

    @context(ValueTypeCtx)
    def value_type(self):
        return ValueTypeCtx(self.type)

    @property
    def type(self) -> T.Optional[GirType]:
        return self.children[TypeName][0].gir_type

    @property
    def type_complete(self) -> bool:
        return True

    @validate()
    def cast_makes_sense(self):
        if self.type is None or self.lhs.type is None:
            return

        if not self.type.assignable_to(self.lhs.type):
            raise CompileError(
                f"Invalid cast. No instance of {self.lhs.type.full_name} can be an instance of {self.type.full_name}."
            )

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

    @docs("as")
    def ref_docs(self):
        return get_docs_section("Syntax CastExpression")


class ClosureArg(AstNode):
    grammar = Expression

    @property
    def expr(self) -> Expression:
        return self.children[Expression][0]

    @context(ValueTypeCtx)
    def value_type(self) -> ValueTypeCtx:
        return ValueTypeCtx(None)


class ClosureExpr(ExprBase):
    grammar = [
        Optional(["$", UseLiteral("extern", True)]),
        UseIdent("name"),
        "(",
        Delimited(ClosureArg, ","),
        ")",
    ]

    @property
    def type(self) -> T.Optional[GirType]:
        if isinstance(self.rhs, CastExpr):
            return self.rhs.type
        else:
            return None

    @property
    def closure_name(self) -> str:
        return self.tokens["name"]

    @property
    def args(self) -> T.List[ClosureArg]:
        return self.children[ClosureArg]

    @validate()
    def cast_to_return_type(self):
        if not isinstance(self.rhs, CastExpr):
            raise CompileError(
                "Closure expression must be cast to the closure's return type"
            )

    @validate()
    def builtin_exists(self):
        if not self.tokens["extern"]:
            raise CompileError(f"{self.closure_name} is not a builtin function")

    @docs("name")
    def ref_docs(self):
        return get_docs_section("Syntax ClosureExpression")


expr.children = [
    AnyOf(ClosureExpr, LiteralExpr, ["(", Expression, ")"]),
    ZeroOrMore(AnyOf(LookupOp, CastExpr)),
]


@decompiler("lookup", skip_children=True, cdata=True)
def decompile_lookup(
    ctx: DecompileCtx, gir: gir.GirContext, cdata: str, name: str, type: str
):
    if t := ctx.type_by_cname(type):
        type = decompile.full_name(t)
    else:
        type = "$" + type

    assert ctx.current_node is not None

    constant = None
    if len(ctx.current_node.children) == 0:
        constant = cdata
    elif (
        len(ctx.current_node.children) == 1
        and ctx.current_node.children[0].tag == "constant"
    ):
        constant = ctx.current_node.children[0].cdata

    if constant is not None:
        if constant == ctx.template_class:
            ctx.print("template." + name)
        else:
            ctx.print(constant + "." + name)
        return
    else:
        for child in ctx.current_node.children:
            decompile.decompile_element(ctx, gir, child)

    ctx.print(f" as <{type}>.{name}")


@decompiler("constant", cdata=True)
def decompile_constant(
    ctx: DecompileCtx, gir: gir.GirContext, cdata: str, type: T.Optional[str] = None
):
    if type is None:
        if cdata == ctx.template_class:
            ctx.print("template")
        else:
            ctx.print(cdata)
    else:
        _, string = ctx.decompile_value(cdata, ctx.type_by_cname(type))
        ctx.print(string)


@decompiler("closure", skip_children=True)
def decompile_closure(ctx: DecompileCtx, gir: gir.GirContext, function: str, type: str):
    if t := ctx.type_by_cname(type):
        type = decompile.full_name(t)
    else:
        type = "$" + type

    ctx.print(f"${function}(")

    assert ctx.current_node is not None
    for i, node in enumerate(ctx.current_node.children):
        decompile_element(ctx, gir, node)

        assert ctx.current_node is not None
        if i < len(ctx.current_node.children) - 1:
            ctx.print(", ")

    ctx.end_block_with(f") as <{type}>")
