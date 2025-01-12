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


import unittest

from blueprintcompiler.errors import PrintableError
from blueprintcompiler.tokenizer import Token, TokenType, tokenize


class TestTokenizer(unittest.TestCase):
    def assert_tokenize(self, string: str, expect: [Token]):
        try:
            tokens = tokenize(string)
            self.assertEqual(len(tokens), len(expect))
            for token, (type, token_str) in zip(tokens, expect):
                self.assertEqual(token.type, type)
                self.assertEqual(str(token), token_str)
        except PrintableError as e:  # pragma: no cover
            e.pretty_print("<test input>", string)
            raise e

    def test_basic(self):
        self.assert_tokenize(
            "ident(){}; \n <<+>>*/=",
            [
                (TokenType.IDENT, "ident"),
                (TokenType.PUNCTUATION, "("),
                (TokenType.PUNCTUATION, ")"),
                (TokenType.PUNCTUATION, "{"),
                (TokenType.PUNCTUATION, "}"),
                (TokenType.PUNCTUATION, ";"),
                (TokenType.WHITESPACE, " \n "),
                (TokenType.OP, "<<"),
                (TokenType.OP, "+"),
                (TokenType.OP, ">>"),
                (TokenType.OP, "*"),
                (TokenType.OP, "/"),
                (TokenType.OP, "="),
                (TokenType.EOF, ""),
            ],
        )

    def test_quotes(self):
        self.assert_tokenize(
            r'"this is a \n string""this is \\another \"string\""',
            [
                (TokenType.QUOTED, r'"this is a \n string"'),
                (TokenType.QUOTED, r'"this is \\another \"string\""'),
                (TokenType.EOF, ""),
            ],
        )

    def test_comments(self):
        self.assert_tokenize(
            "/* \n \\n COMMENT /* */",
            [
                (TokenType.COMMENT, "/* \n \\n COMMENT /* */"),
                (TokenType.EOF, ""),
            ],
        )
        self.assert_tokenize(
            "line // comment\nline",
            [
                (TokenType.IDENT, "line"),
                (TokenType.WHITESPACE, " "),
                (TokenType.COMMENT, "// comment"),
                (TokenType.WHITESPACE, "\n"),
                (TokenType.IDENT, "line"),
                (TokenType.EOF, ""),
            ],
        )
