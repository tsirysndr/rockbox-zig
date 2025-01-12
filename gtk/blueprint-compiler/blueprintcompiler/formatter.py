# formatter.py
#
# Copyright 2023 Gregor Niehl <gregorniehl@web.de>
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
from enum import Enum

from . import tokenizer, utils
from .tokenizer import TokenType

OPENING_TOKENS = ("{", "[")
CLOSING_TOKENS = ("}", "]")

NEWLINE_AFTER = tuple(";") + OPENING_TOKENS + CLOSING_TOKENS

NO_WHITESPACE_BEFORE = (",", ":", "::", ";", ")", ".", ">", "]", "=")
NO_WHITESPACE_AFTER = ("C_", "_", "(", ".", "$", "<", "::", "[", "=")

# NO_WHITESPACE_BEFORE takes precedence over WHITESPACE_AFTER
WHITESPACE_AFTER = (":", ",", ">", ")", "|", "=>")
WHITESPACE_BEFORE = ("{", "|")


class LineType(Enum):
    STATEMENT = 0
    BLOCK_OPEN = 1
    BLOCK_CLOSE = 2
    CHILD_TYPE = 3
    COMMENT = 4


def format(data, tab_size=2, insert_space=True):
    indent_levels = 0
    tokens = tokenizer.tokenize(data)
    end_str = ""
    last_not_whitespace = tokens[0]
    current_line = ""
    prev_line_type = None
    is_child_type = False
    indent_item = " " * tab_size if insert_space else "\t"
    watch_parentheses = False
    parentheses_balance = 0
    bracket_tracker = [None]
    last_whitespace_contains_newline = False

    def commit_current_line(
        line_type=prev_line_type, redo_whitespace=False, newlines_before=1
    ):
        nonlocal end_str, current_line, prev_line_type

        indent_whitespace = indent_levels * indent_item
        whitespace_to_add = "\n" + indent_whitespace

        if redo_whitespace or newlines_before != 1:
            end_str = end_str.strip() + "\n" * newlines_before
            if newlines_before > 0:
                end_str += indent_whitespace

        end_str += current_line + whitespace_to_add

        current_line = ""
        prev_line_type = line_type

    for item in tokens:
        str_item = str(item)

        if item.type == TokenType.WHITESPACE:
            last_whitespace_contains_newline = "\n" in str_item
            continue

        whitespace_required = (
            str_item in WHITESPACE_BEFORE
            or str(last_not_whitespace) in WHITESPACE_AFTER
            or (str_item == "(" and end_str.endswith(": bind"))
        )
        whitespace_blockers = (
            str_item in NO_WHITESPACE_BEFORE
            or str(last_not_whitespace) in NO_WHITESPACE_AFTER
            or (str_item == "<" and str(last_not_whitespace) == "typeof")
        )

        this_or_last_is_ident = TokenType.IDENT in (item.type, last_not_whitespace.type)
        current_line_is_empty = len(current_line) == 0
        is_function = str_item == "(" and not re.match(
            r"^([A-Za-z_\-])+(: bind)?$", current_line
        )

        any_blockers = whitespace_blockers or current_line_is_empty or is_function
        if (whitespace_required or this_or_last_is_ident) and not any_blockers:
            current_line += " "

        current_line += str_item

        if str_item in ("[", "("):
            bracket_tracker.append(str_item)
        elif str_item in ("]", ")"):
            bracket_tracker.pop()

        needs_newline_treatment = (
            str_item in NEWLINE_AFTER or item.type == TokenType.COMMENT
        )
        if needs_newline_treatment:
            if str_item in OPENING_TOKENS:
                list_or_child_type = str_item == "["
                if list_or_child_type:
                    is_child_type = current_line.startswith("[")

                    if is_child_type:
                        if str(last_not_whitespace) not in OPENING_TOKENS:
                            end_str = (
                                end_str.strip() + "\n\n" + (indent_item * indent_levels)
                            )
                        last_not_whitespace = item
                        continue

                indent_levels += 1
                keep_same_indent = prev_line_type not in (
                    LineType.CHILD_TYPE,
                    LineType.COMMENT,
                    LineType.BLOCK_OPEN,
                )
                if keep_same_indent:
                    end_str = (
                        end_str.strip() + "\n\n" + indent_item * (indent_levels - 1)
                    )
                commit_current_line(LineType.BLOCK_OPEN)

            elif str_item == "]" and is_child_type:
                commit_current_line(LineType.CHILD_TYPE, False)
                is_child_type = False

            elif str_item in CLOSING_TOKENS:
                if str_item == "]" and last_not_whitespace != ",":
                    current_line = current_line[:-1]
                    commit_current_line()
                    current_line = "]"
                elif str(last_not_whitespace) in OPENING_TOKENS:
                    end_str = end_str.strip()
                    commit_current_line(LineType.BLOCK_CLOSE, True, 0)

                indent_levels -= 1
                commit_current_line(LineType.BLOCK_CLOSE, True)

            elif str_item == ";":
                line_type = LineType.STATEMENT
                newlines = 1

                if len(current_line) == 1:
                    newlines = 0
                    line_type = LineType.BLOCK_CLOSE
                elif prev_line_type == LineType.BLOCK_CLOSE:
                    newlines = 2

                commit_current_line(line_type, newlines_before=newlines)

            elif item.type == TokenType.COMMENT:
                require_extra_newline = (
                    LineType.BLOCK_CLOSE,
                    LineType.STATEMENT,
                    LineType.COMMENT,
                )

                single_line_comment = str_item.startswith("//")
                newlines = 1
                if single_line_comment:
                    if not str_item.startswith("// "):
                        current_line = f"// {current_line[2:]}"

                    if not last_whitespace_contains_newline:
                        current_line = " " + current_line
                        newlines = 0
                    elif prev_line_type == LineType.BLOCK_CLOSE:
                        newlines = 2

                elif prev_line_type in require_extra_newline:
                    newlines = 2

                commit_current_line(LineType.COMMENT, newlines_before=newlines)

            else:
                commit_current_line()

        elif str_item == "(" and (
            re.match(r"^([A-Za-z_\-])+\s*\(", current_line) or watch_parentheses
        ):
            watch_parentheses = True
            parentheses_balance += 1

        elif str_item == ")" and watch_parentheses:
            parentheses_balance -= 1
            all_parentheses_closed = parentheses_balance == 0
            if all_parentheses_closed:
                commit_current_line(
                    newlines_before=2 if prev_line_type == LineType.BLOCK_CLOSE else 1
                )
                watch_parentheses = False

        tracker_is_empty = len(bracket_tracker) > 0
        if tracker_is_empty:
            last_in_tracker = bracket_tracker[-1]
            is_list_comma = last_in_tracker == "[" and str_item == ","
            if is_list_comma:
                last_was_list_item = end_str.strip()[-1] not in ("[", ",")
                if last_was_list_item:
                    end_str = end_str.strip()
                commit_current_line()

        last_not_whitespace = item
        last_whitespace_contains_newline = False

    return end_str.strip() + "\n"
