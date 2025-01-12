# tokenizer.py
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


import re
import typing as T
from dataclasses import dataclass
from enum import Enum

from . import utils


class TokenType(Enum):
    EOF = 0
    IDENT = 1
    QUOTED = 2
    NUMBER = 3
    OP = 4
    WHITESPACE = 5
    COMMENT = 6
    PUNCTUATION = 7


_tokens = [
    (TokenType.IDENT, r"[A-Za-z_][\d\w\-_]*"),
    (TokenType.QUOTED, r'"(\\(.|\n)|[^\\"\n])*"'),
    (TokenType.QUOTED, r"'(\\(.|\n)|[^\\'\n])*'"),
    (TokenType.NUMBER, r"0x[A-Za-z0-9_]+"),
    (TokenType.NUMBER, r"[\d_]+(\.[\d_]+)?"),
    (TokenType.NUMBER, r"\.[\d_]+"),
    (TokenType.WHITESPACE, r"\s+"),
    (TokenType.COMMENT, r"\/\*[\s\S]*?\*\/"),
    (TokenType.COMMENT, r"\/\/[^\n]*"),
    (TokenType.OP, r"\$|<<|>>|=>|::|<|>|:=|\.|\|\||\||\+|\-|\*|=|:|/"),
    (TokenType.PUNCTUATION, r"\(|\)|\{|\}|;|\[|\]|\,"),
]
_TOKENS = [(type, re.compile(regex)) for (type, regex) in _tokens]


class Token:
    def __init__(self, type: TokenType, start: int, end: int, string: str):
        self.type = type
        self.start = start
        self.end = end
        self.string = string

    def __str__(self) -> str:
        return self.string[self.start : self.end]

    @property
    def range(self) -> "Range":
        return Range(self.start, self.end, self.string)

    def get_number(self) -> T.Union[int, float]:
        from .errors import CompileError, CompilerBugError

        if self.type != TokenType.NUMBER:
            raise CompilerBugError()

        string = str(self).replace("_", "")
        try:
            if string.startswith("0x"):
                return int(string, 16)
            elif "." in string:
                return float(string)
            else:
                return int(string)
        except:
            raise CompileError(f"{str(self)} is not a valid number literal", self.range)


def _tokenize(ui_ml: str):
    from .errors import CompileError

    i = 0
    while i < len(ui_ml):
        matched = False
        for type, regex in _TOKENS:
            match = regex.match(ui_ml, i)

            if match is not None:
                yield Token(type, match.start(), match.end(), ui_ml)
                i = match.end()
                matched = True
                break

        if not matched:
            raise CompileError(
                "Could not determine what kind of syntax is meant here",
                Range(i, i, ui_ml),
            )

    yield Token(TokenType.EOF, i, i, ui_ml)


def tokenize(data: str) -> T.List[Token]:
    return list(_tokenize(data))


@dataclass
class Range:
    start: int
    end: int
    original_text: str

    @property
    def length(self) -> int:
        return self.end - self.start

    @property
    def text(self) -> str:
        return self.original_text[self.start : self.end]

    @property
    def with_trailing_newline(self) -> "Range":
        if len(self.original_text) > self.end and self.original_text[self.end] == "\n":
            return Range(self.start, self.end + 1, self.original_text)
        else:
            return self

    @staticmethod
    def join(a: T.Optional["Range"], b: T.Optional["Range"]) -> T.Optional["Range"]:
        if a is None:
            return b
        if b is None:
            return a
        return Range(min(a.start, b.start), max(a.end, b.end), a.original_text)

    def __contains__(self, other: T.Union[int, "Range"]) -> bool:
        if isinstance(other, int):
            return self.start <= other <= self.end
        else:
            return self.start <= other.start and self.end >= other.end

    def to_json(self):
        return utils.idxs_to_range(self.start, self.end, self.original_text)

    def overlaps(self, other: "Range") -> bool:
        return not (self.end < other.start or self.start > other.end)
