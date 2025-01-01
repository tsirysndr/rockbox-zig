# lsp_enums.py
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


import enum
import json
import os
import typing as T
from dataclasses import dataclass, field

from .errors import *
from .tokenizer import Range
from .utils import *


class TextDocumentSyncKind(enum.IntEnum):
    None_ = 0
    Full = 1
    Incremental = 2


class CompletionItemTag(enum.IntEnum):
    Deprecated = 1


class InsertTextFormat(enum.IntEnum):
    PlainText = 1
    Snippet = 2


class CompletionItemKind(enum.IntEnum):
    Text = 1
    Method = 2
    Function = 3
    Constructor = 4
    Field = 5
    Variable = 6
    Class = 7
    Interface = 8
    Module = 9
    Property = 10
    Unit = 11
    Value = 12
    Enum = 13
    Keyword = 14
    Snippet = 15
    Color = 16
    File = 17
    Reference = 18
    Folder = 19
    EnumMember = 20
    Constant = 21
    Struct = 22
    Event = 23
    Operator = 24
    TypeParameter = 25


class ErrorCode(enum.IntEnum):
    RequestFailed = -32803


@dataclass
class Completion:
    label: str
    kind: CompletionItemKind
    signature: T.Optional[str] = None
    deprecated: bool = False
    sort_text: T.Optional[str] = None
    docs: T.Optional[str] = None
    text: T.Optional[str] = None
    snippet: T.Optional[str] = None
    detail: T.Optional[str] = None

    def to_json(self, snippets: bool):
        insert_text = self.text or self.label
        insert_text_format = InsertTextFormat.PlainText
        if snippets and self.snippet:
            insert_text = self.snippet
            insert_text_format = InsertTextFormat.Snippet

        result = {
            "label": self.label,
            "kind": self.kind,
            "tags": [CompletionItemTag.Deprecated] if self.deprecated else None,
            # https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionItemLabelDetails
            "labelDetails": ({"detail": self.signature} if self.signature else None),
            "documentation": (
                {
                    "kind": "markdown",
                    "value": self.docs,
                }
                if self.docs
                else None
            ),
            "deprecated": self.deprecated,
            "sortText": self.sort_text,
            "insertText": insert_text,
            "insertTextFormat": insert_text_format,
            "detail": self.detail if self.detail else None,
        }
        return {k: v for k, v in result.items() if v is not None}


class SemanticTokenType(enum.IntEnum):
    EnumMember = 0


class DiagnosticSeverity(enum.IntEnum):
    Error = 1
    Warning = 2
    Information = 3
    Hint = 4


class DiagnosticTag(enum.IntEnum):
    Unnecessary = 1
    Deprecated = 2


@dataclass
class SemanticToken:
    start: int
    end: int
    type: SemanticTokenType


class SymbolKind(enum.IntEnum):
    File = 1
    Module = 2
    Namespace = 3
    Package = 4
    Class = 5
    Method = 6
    Property = 7
    Field = 8
    Constructor = 9
    Enum = 10
    Interface = 11
    Function = 12
    Variable = 13
    Constant = 14
    String = 15
    Number = 16
    Boolean = 17
    Array = 18
    Object = 19
    Key = 20
    Null = 21
    EnumMember = 22
    Struct = 23
    Event = 24
    Operator = 25
    TypeParameter = 26


@dataclass
class DocumentSymbol:
    name: str
    kind: SymbolKind
    range: Range
    selection_range: Range
    detail: T.Optional[str] = None
    children: T.List["DocumentSymbol"] = field(default_factory=list)


@dataclass
class LocationLink:
    origin_selection_range: Range
    target_range: Range
    target_selection_range: Range

    def to_json(self, target_uri: str):
        return {
            "originSelectionRange": self.origin_selection_range.to_json(),
            "targetUri": target_uri,
            "targetRange": self.target_range.to_json(),
            "targetSelectionRange": self.target_selection_range.to_json(),
        }


@dataclass
class TextEdit:
    range: Range
    newText: str

    def to_json(self):
        return {"range": self.range.to_json(), "newText": self.newText}


_docs_sections: T.Optional[dict[str, T.Any]] = None


def get_docs_section(section_name: str) -> T.Optional[str]:
    global _docs_sections

    if _docs_sections is None:
        try:
            with open(
                os.path.join(os.path.dirname(__file__), "reference_docs.json")
            ) as f:
                _docs_sections = json.load(f)
        except FileNotFoundError:
            _docs_sections = {}

    if section := _docs_sections.get(section_name):
        content = section["content"]
        link = section["link"]
        content += f"\n\n---\n\n[Online documentation]({link})"
        return content
    else:
        return None
