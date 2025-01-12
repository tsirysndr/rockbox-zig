# gtk_combo_box_text.py
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


from .common import *
from .contexts import ValueTypeCtx
from .gobject_object import ObjectContent, validate_parent_type
from .values import StringValue


class Item(AstNode):
    grammar = [
        Optional([UseIdent("name"), ":"]),
        StringValue,
    ]

    @property
    def name(self) -> T.Optional[str]:
        return self.tokens["name"]

    @property
    def value(self) -> StringValue:
        return self.children[StringValue][0]

    @property
    def document_symbol(self) -> DocumentSymbol:
        return DocumentSymbol(
            self.value.range.text,
            SymbolKind.String,
            self.range,
            self.value.range,
            self.name,
        )

    @validate("name")
    def unique_in_parent(self):
        if self.name is not None:
            self.validate_unique_in_parent(
                f"Duplicate item '{self.name}'", lambda x: x.name == self.name
            )

    @docs("name")
    def ref_docs(self):
        return get_docs_section("Syntax ExtComboBoxItems")


class ExtComboBoxItems(AstNode):
    grammar = [
        Keyword("items"),
        "[",
        Delimited(Item, ","),
        "]",
    ]

    @property
    def document_symbol(self) -> DocumentSymbol:
        return DocumentSymbol(
            "items",
            SymbolKind.Array,
            self.range,
            self.group.tokens["items"].range,
        )

    @validate("items")
    def container_is_combo_box_text(self):
        validate_parent_type(self, "Gtk", "ComboBoxText", "combo box items")

    @validate("items")
    def unique_in_parent(self):
        self.validate_unique_in_parent("Duplicate items block")

    @docs("items")
    def ref_docs(self):
        return get_docs_section("Syntax ExtComboBoxItems")


@completer(
    applies_in=[ObjectContent],
    applies_in_subclass=("Gtk", "ComboBoxText"),
    matches=new_statement_patterns,
)
def items_completer(lsp, ast_node, match_variables):
    yield Completion("items", CompletionItemKind.Snippet, snippet="items [$0]")


@decompiler("items", parent_type="Gtk.ComboBoxText")
def decompile_items(ctx: DecompileCtx, gir: gir.GirContext):
    ctx.print("items [")


@decompiler("item", parent_type="Gtk.ComboBoxText", cdata=True)
def decompile_item(
    ctx: DecompileCtx,
    gir: gir.GirContext,
    cdata: str,
    id: T.Optional[str] = None,
    translatable="false",
    comments=None,
    context=None,
):
    comments, translatable = decompile_translatable(
        cdata, translatable, context, comments
    )
    if comments:
        ctx.print(comments)
    if id:
        ctx.print(f"{id}: ")
    ctx.print(translatable)
    ctx.print(",")
