# errors.py
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

import sys
import traceback
import typing as T
from dataclasses import dataclass

from . import utils
from .tokenizer import Range
from .utils import Colors


class PrintableError(Exception):
    """Parent class for errors that can be pretty-printed for the user, e.g.
    compilation warnings and errors."""

    def pretty_print(self, filename, code, stream=sys.stdout):
        raise NotImplementedError()


@dataclass
class ErrorReference:
    range: Range
    message: str


class CompileError(PrintableError):
    """A PrintableError with a start/end position and optional hints"""

    category = "error"
    color = Colors.RED

    def __init__(
        self,
        message: str,
        range: T.Optional[Range] = None,
        did_you_mean: T.Optional[T.Tuple[str, T.List[str]]] = None,
        hints: T.Optional[T.List[str]] = None,
        actions: T.Optional[T.List["CodeAction"]] = None,
        fatal: bool = False,
        references: T.Optional[T.List[ErrorReference]] = None,
    ) -> None:
        super().__init__(message)

        self.message = message
        self.range = range
        self.hints = hints or []
        self.actions = actions or []
        self.references = references or []
        self.fatal = fatal

        if did_you_mean is not None:
            self._did_you_mean(*did_you_mean)

    def hint(self, hint: str) -> "CompileError":
        self.hints.append(hint)
        return self

    def _did_you_mean(self, word: str, options: T.List[str]) -> None:
        if word.replace("_", "-") in options:
            self.hint(f"use '-', not '_': `{word.replace('_', '-')}`")
            return

        recommend = utils.did_you_mean(word, options)
        if recommend is not None:
            if word.casefold() == recommend.casefold():
                self.hint(f"Did you mean `{recommend}` (note the capitalization)?")
            else:
                self.hint(f"Did you mean `{recommend}`?")
            self.actions.append(CodeAction(f"Change to `{recommend}`", recommend))
        else:
            self.hint("Did you check your spelling?")
            self.hint("Are your dependencies up to date?")

    def pretty_print(self, filename: str, code: str, stream=sys.stdout) -> None:
        assert self.range is not None

        line_num, col_num = utils.idx_to_pos(self.range.start + 1, code)
        end_line_num, end_col_num = utils.idx_to_pos(self.range.end + 1, code)
        line = code.splitlines(True)[line_num] if code != "" else ""

        # Display 1-based line numbers
        line_num += 1
        end_line_num += 1

        n_spaces = col_num - 1
        n_carets = (
            (end_col_num - col_num)
            if line_num == end_line_num
            else (len(line) - n_spaces - 1)
        )

        n_spaces += line.count("\t", 0, col_num)
        n_carets += line.count("\t", col_num, col_num + n_carets)
        line = line.replace("\t", "  ")

        stream.write(
            f"""{self.color}{Colors.BOLD}{self.category}: {self.message}{Colors.CLEAR}
at {filename} line {line_num} column {col_num}:
{Colors.FAINT}{line_num :>4} |{Colors.CLEAR}{line.rstrip()}\n     {Colors.FAINT}|{" "*n_spaces}{"^"*n_carets}{Colors.CLEAR}\n"""
        )

        for hint in self.hints:
            stream.write(f"{Colors.FAINT}hint: {hint}{Colors.CLEAR}\n")

        for i, action in enumerate(self.actions):
            old = (
                action.edit_range.text
                if action.edit_range is not None
                else self.range.text
            )

            if old == "":
                stream.write(
                    f"suggestion: insert {Colors.GREEN}{action.replace_with}{Colors.CLEAR}\n"
                )
            elif action.replace_with == "":
                stream.write(f"suggestion: remove {Colors.RED}{old}{Colors.CLEAR}\n")
            else:
                stream.write(
                    f"suggestion: replace {Colors.RED}{old}{Colors.CLEAR} with {Colors.GREEN}{action.replace_with}{Colors.CLEAR}\n"
                )

        for ref in self.references:
            line_num, col_num = utils.idx_to_pos(ref.range.start + 1, code)
            line = code.splitlines(True)[line_num]
            line_num += 1

            stream.write(
                f"""{Colors.FAINT}note: {ref.message}:
at {filename} line {line_num} column {col_num}:
{Colors.FAINT}{line_num :>4} |{line.rstrip()}\n     {Colors.FAINT}|{" "*(col_num-1)}^{Colors.CLEAR}\n"""
            )

        stream.write("\n")


class CompileWarning(CompileError):
    category = "warning"
    color = Colors.YELLOW


class DeprecatedWarning(CompileWarning):
    pass


class UnusedWarning(CompileWarning):
    pass


class UpgradeWarning(CompileWarning):
    category = "upgrade"
    color = Colors.PURPLE


class UnexpectedTokenError(CompileError):
    def __init__(self, range: Range) -> None:
        super().__init__("Unexpected tokens", range)


@dataclass
class CodeAction:
    title: str
    replace_with: str
    edit_range: T.Optional[Range] = None


class MultipleErrors(PrintableError):
    """If multiple errors occur during compilation, they can be collected into
    a list and re-thrown using the MultipleErrors exception. It will
    pretty-print all of the errors and a count of how many errors there are."""

    def __init__(self, errors: T.List[CompileError]) -> None:
        super().__init__()
        self.errors = errors

    def pretty_print(self, filename, code, stream=sys.stdout) -> None:
        for error in self.errors:
            error.pretty_print(filename, code, stream)
        if len(self.errors) != 1:
            print(f"{len(self.errors)} errors")


class CompilerBugError(Exception):
    """Emitted on assertion errors"""


def assert_true(truth: bool, message: T.Optional[str] = None):
    if not truth:
        raise CompilerBugError(message)


def report_bug():  # pragma: no cover
    """Report an error and ask people to report it."""

    from . import main

    print(traceback.format_exc())
    print(f"Arguments: {sys.argv}")
    print(f"Version: {main.VERSION}\n")
    print(
        f"""{Colors.BOLD}{Colors.RED}***** COMPILER BUG *****
The blueprint-compiler program has crashed. Please report the above stacktrace,
along with the input file(s) if possible, on GitLab:
{Colors.BOLD}{Colors.BLUE}{Colors.UNDERLINE}https://gitlab.gnome.org/jwestman/blueprint-compiler/-/issues/new?issue
{Colors.CLEAR}"""
    )

    sys.exit(1)
