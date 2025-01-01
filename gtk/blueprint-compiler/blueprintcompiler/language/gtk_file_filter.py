# gtk_file_filter.py
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
from .gobject_object import ObjectContent, validate_parent_type


class Filters(AstNode):
    @property
    def document_symbol(self) -> DocumentSymbol:
        return DocumentSymbol(
            self.tokens["tag_name"],
            SymbolKind.Array,
            self.range,
            self.group.tokens["tag_name"].range,
        )

    @validate()
    def container_is_file_filter(self):
        validate_parent_type(self, "Gtk", "FileFilter", "file filter properties")

    @validate("tag_name")
    def unique_in_parent(self):
        self.validate_unique_in_parent(
            f"Duplicate {self.tokens['tag_name']} block",
            check=lambda child: child.tokens["tag_name"] == self.tokens["tag_name"],
        )

    @docs("tag_name")
    def ref_docs(self):
        return get_docs_section("Syntax ExtFileFilter")


class FilterString(AstNode):
    @property
    def item(self) -> str:
        return self.tokens["name"]

    @property
    def document_symbol(self) -> DocumentSymbol:
        return DocumentSymbol(
            self.item,
            SymbolKind.String,
            self.range,
            self.group.tokens["name"].range,
        )

    @validate()
    def unique_in_parent(self):
        self.validate_unique_in_parent(
            f"Duplicate {self.tokens['tag_name']} '{self.item}'",
            check=lambda child: child.item == self.item,
        )


def create_node(tag_name: str, singular: str):
    return Group(
        Filters,
        [
            UseExact("tag_name", tag_name),
            "[",
            Delimited(
                Group(
                    FilterString,
                    [
                        UseQuoted("name"),
                        UseLiteral("tag_name", singular),
                    ],
                ),
                ",",
            ),
            "]",
        ],
    )


ext_file_filter_mime_types = create_node("mime-types", "mime-type")
ext_file_filter_patterns = create_node("patterns", "pattern")
ext_file_filter_suffixes = create_node("suffixes", "suffix")


@completer(
    applies_in=[ObjectContent],
    applies_in_subclass=("Gtk", "FileFilter"),
    matches=new_statement_patterns,
)
def file_filter_completer(lsp, ast_node, match_variables):
    yield Completion(
        "mime-types", CompletionItemKind.Snippet, snippet='mime-types ["$0"]'
    )
    yield Completion("patterns", CompletionItemKind.Snippet, snippet='patterns ["$0"]')
    yield Completion("suffixes", CompletionItemKind.Snippet, snippet='suffixes ["$0"]')


@decompiler("mime-types")
def decompile_mime_types(ctx, gir):
    ctx.print("mime-types [")


@decompiler("mime-type", cdata=True)
def decompile_mime_type(ctx, gir, cdata):
    ctx.print(f"{escape_quote(cdata)},")


@decompiler("patterns")
def decompile_patterns(ctx, gir):
    ctx.print("patterns [")


@decompiler("pattern", cdata=True)
def decompile_pattern(ctx, gir, cdata):
    ctx.print(f"{escape_quote(cdata)},")


@decompiler("suffixes")
def decompile_suffixes(ctx, gir):
    ctx.print("suffixes [")


@decompiler("suffix", cdata=True)
def decompile_suffix(ctx, gir, cdata):
    ctx.print(f"{escape_quote(cdata)},")
