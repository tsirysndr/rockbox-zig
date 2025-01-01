# test_formatter.py
#
# Copyright 2023 James Westman <james@jwestman.net>
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
from pathlib import Path

from blueprintcompiler import formatter


class TestFormatter(unittest.TestCase):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.maxDiff = None

    def assert_format_test(self, input_file, expected_file):
        print("assert_format_test({}, {})".format(input_file, expected_file))
        with open((Path(__file__).parent / f"formatting/{input_file}").resolve()) as f:
            input_data = f.read()
        with open(
            (Path(__file__).parent / f"formatting/{expected_file}").resolve()
        ) as f:
            expected = f.read()

        actual = formatter.format(input_data)
        self.assertEqual(actual, expected)

    def test_formatter(self):
        self.assert_format_test("in1.blp", "out.blp")
        self.assert_format_test("in2.blp", "out.blp")
        self.assert_format_test("correct1.blp", "correct1.blp")
        self.assert_format_test("string_in.blp", "string_out.blp")
