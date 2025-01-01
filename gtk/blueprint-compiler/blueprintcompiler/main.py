# main.py
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


import argparse
import difflib
import os
import sys
import typing as T

from . import formatter, interactive_port, parser, tokenizer
from .decompiler import decompile_string
from .errors import CompileError, CompilerBugError, PrintableError, report_bug
from .gir import add_typelib_search_path
from .lsp import LanguageServer
from .outputs import XmlOutput
from .utils import Colors

VERSION = "uninstalled"
LIBDIR = None


class BlueprintApp:
    def main(self):
        self.parser = argparse.ArgumentParser()
        self.subparsers = self.parser.add_subparsers(metavar="command")
        self.parser.set_defaults(func=self.cmd_help)

        compile = self.add_subcommand(
            "compile", "Compile blueprint files", self.cmd_compile
        )
        compile.add_argument("--output", dest="output", default="-")
        compile.add_argument("--typelib-path", nargs="?", action="append")
        compile.add_argument(
            "input", metavar="filename", default=sys.stdin, type=argparse.FileType("r")
        )

        batch_compile = self.add_subcommand(
            "batch-compile",
            "Compile many blueprint files at once",
            self.cmd_batch_compile,
        )
        batch_compile.add_argument("output_dir", metavar="output-dir")
        batch_compile.add_argument("input_dir", metavar="input-dir")
        batch_compile.add_argument("--typelib-path", nargs="?", action="append")
        batch_compile.add_argument(
            "inputs",
            nargs="+",
            metavar="filenames",
            default=sys.stdin,
            type=argparse.FileType("r"),
        )

        format = self.add_subcommand(
            "format", "Format given blueprint files", self.cmd_format
        )
        format.add_argument(
            "-f",
            "--fix",
            help="Apply the edits to the files",
            default=False,
            action="store_true",
        )
        format.add_argument(
            "-t",
            "--tabs",
            help="Use tabs instead of spaces",
            default=False,
            action="store_true",
        )
        format.add_argument(
            "-s",
            "--spaces-num",
            help="How many spaces should be used per indent",
            default=2,
            type=int,
        )
        format.add_argument(
            "-n",
            "--no-diff",
            help="Do not print a full diff of the changes",
            default=False,
            action="store_true",
        )
        format.add_argument(
            "inputs",
            nargs="+",
            metavar="filenames",
        )

        decompile = self.add_subcommand(
            "decompile", "Convert .ui XML files to blueprint", self.cmd_decompile
        )
        decompile.add_argument("--output", dest="output", default="-")
        decompile.add_argument("--typelib-path", nargs="?", action="append")
        decompile.add_argument(
            "input", metavar="filename", default=sys.stdin, type=argparse.FileType("r")
        )

        port = self.add_subcommand("port", "Interactive porting tool", self.cmd_port)

        lsp = self.add_subcommand(
            "lsp", "Run the language server (for internal use by IDEs)", self.cmd_lsp
        )

        self.add_subcommand("help", "Show this message", self.cmd_help)

        self.parser.add_argument("--version", action="version", version=VERSION)

        try:
            opts = self.parser.parse_args()
            opts.func(opts)
        except SystemExit as e:
            raise e
        except KeyboardInterrupt:
            print(f"\n\n{Colors.RED}{Colors.BOLD}Interrupted.{Colors.CLEAR}")
        except EOFError:
            print(f"\n\n{Colors.RED}{Colors.BOLD}Interrupted.{Colors.CLEAR}")
        except:
            report_bug()

    def add_subcommand(self, name: str, help: str, func):
        parser = self.subparsers.add_parser(name, help=help)
        parser.set_defaults(func=func)
        return parser

    def cmd_help(self, opts):
        self.parser.print_help()

    def cmd_compile(self, opts):
        if opts.typelib_path != None:
            for typelib_path in opts.typelib_path:
                add_typelib_search_path(typelib_path)

        data = opts.input.read()
        try:
            xml, warnings = self._compile(data)

            for warning in warnings:
                warning.pretty_print(opts.input.name, data, stream=sys.stderr)

            if opts.output == "-":
                print(xml)
            else:
                with open(opts.output, "w") as file:
                    file.write(xml)
        except PrintableError as e:
            e.pretty_print(opts.input.name, data, stream=sys.stderr)
            sys.exit(1)

    def cmd_batch_compile(self, opts):
        if opts.typelib_path != None:
            for typelib_path in opts.typelib_path:
                add_typelib_search_path(typelib_path)

        for file in opts.inputs:
            data = file.read()
            file_abs = os.path.abspath(file.name)
            input_dir_abs = os.path.abspath(opts.input_dir)

            try:
                if not os.path.commonpath([file_abs, input_dir_abs]):
                    print(
                        f"{Colors.RED}{Colors.BOLD}error: input file '{file.name}' is not in input directory '{opts.input_dir}'{Colors.CLEAR}"
                    )
                    sys.exit(1)

                xml, warnings = self._compile(data)

                for warning in warnings:
                    warning.pretty_print(file.name, data, stream=sys.stderr)

                path = os.path.join(
                    opts.output_dir,
                    os.path.relpath(
                        os.path.splitext(file.name)[0] + ".ui", opts.input_dir
                    ),
                )
                os.makedirs(os.path.dirname(path), exist_ok=True)
                with open(path, "w") as file:
                    file.write(xml)
            except PrintableError as e:
                e.pretty_print(file.name, data)
                sys.exit(1)

    def cmd_format(self, opts):
        input_files = []
        missing_files = []
        panic = False
        formatted_files = 0
        skipped_files = 0

        for path in opts.inputs:
            if os.path.isfile(path):
                input_files.append(path)
            elif os.path.isdir(path):
                for root, subfolders, files in os.walk(path):
                    for file in files:
                        if file.endswith(".blp"):
                            input_files.append(os.path.join(root, file))
            else:
                missing_files.append(path)

        for file in input_files:
            with open(file, "r+") as file:
                data = file.read()
                errored = False

                try:
                    self._compile(data)
                except:
                    errored = True

                formatted_str = formatter.format(data, opts.spaces_num, not opts.tabs)

                if data != formatted_str:
                    happened = "Would format"

                    if opts.fix and not errored:
                        file.seek(0)
                        file.truncate()
                        file.write(formatted_str)
                        happened = "Formatted"

                    if not opts.no_diff:
                        diff_lines = []
                        a_lines = data.splitlines(keepends=True)
                        b_lines = formatted_str.splitlines(keepends=True)

                        for line in difflib.unified_diff(
                            a_lines, b_lines, fromfile=file.name, tofile=file.name, n=5
                        ):
                            # Work around https://bugs.python.org/issue2142
                            # See:
                            # https://www.gnu.org/software/diffutils/manual/html_node/Incomplete-Lines.html
                            if line[-1] == "\n":
                                diff_lines.append(line)
                            else:
                                diff_lines.append(line + "\n")
                                diff_lines.append("\\ No newline at end of file\n")

                        print("".join(diff_lines))

                    to_print = Colors.BOLD
                    if errored:
                        to_print += f"{Colors.RED}Skipped {file.name}: Will not overwrite file with compile errors"
                        panic = True
                        skipped_files += 1
                    else:
                        to_print += f"{happened} {file.name}"
                        formatted_files += 1

                    print(to_print)
                    print(Colors.CLEAR)

        missing_num = len(missing_files)
        summary = ""

        if missing_num > 0:
            print(
                f"{Colors.BOLD}{Colors.RED}Could not find files:{Colors.CLEAR}{Colors.BOLD}"
            )
            for path in missing_files:
                print(f"  {path}")
            print(Colors.CLEAR)
            panic = True

        if len(input_files) == 0:
            print(f"{Colors.RED}No Blueprint files found")
            sys.exit(1)

        def would_be(verb):
            return verb if opts.fix else f"would be {verb}"

        def how_many(count, bold=True):
            string = f"{Colors.BLUE}{count} {'files' if count != 1 else 'file'}{Colors.CLEAR}"
            return Colors.BOLD + string + Colors.BOLD if bold else Colors.CLEAR + string

        if formatted_files > 0:
            summary += f"{how_many(formatted_files)} {would_be('formatted')}, "
            panic = panic or not opts.fix

        left_files = len(input_files) - formatted_files - skipped_files
        summary += f"{how_many(left_files, False)} {would_be('left unchanged')}"

        if skipped_files > 0:
            summary += f", {how_many(skipped_files)} {would_be('skipped')}"

        if missing_num > 0:
            summary += f", {how_many(missing_num)} not found"

        print(summary + Colors.CLEAR)

        if panic:
            sys.exit(1)

    def cmd_decompile(self, opts):
        if opts.typelib_path != None:
            for typelib_path in opts.typelib_path:
                add_typelib_search_path(typelib_path)

        data = opts.input.read()
        try:
            decompiled = decompile_string(data)

            if opts.output == "-":
                print(decompiled)
            else:
                with open(opts.output, "w") as file:
                    file.write(decompiled)
        except PrintableError as e:
            e.pretty_print(opts.input.name, data, stream=sys.stderr)
            sys.exit(1)

    def cmd_lsp(self, opts):
        langserv = LanguageServer()
        langserv.run()

    def cmd_port(self, opts):
        interactive_port.run(opts)

    def _compile(self, data: str) -> T.Tuple[str, T.List[CompileError]]:
        tokens = tokenizer.tokenize(data)
        ast, errors, warnings = parser.parse(tokens)

        if errors:
            raise errors
        if ast is None:
            raise CompilerBugError()

        formatter = XmlOutput()

        return formatter.emit(ast), warnings


def main(version, libdir):
    global VERSION, LIBDIR
    VERSION, LIBDIR = version, libdir
    BlueprintApp().main()
