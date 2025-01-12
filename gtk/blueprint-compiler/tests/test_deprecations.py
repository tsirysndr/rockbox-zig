# test_samples.py
#
# Copyright 2024 James Westman <james@jwestman.net>
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

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

from blueprintcompiler import parser, tokenizer
from blueprintcompiler.errors import DeprecatedWarning, PrintableError

# Testing deprecation warnings requires special handling because libraries can add deprecations with new versions,
# causing tests to break if we're not careful.


class TestDeprecations(unittest.TestCase):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.gtkVersion = f"{Gtk.get_major_version()}.{Gtk.get_minor_version()}.{Gtk.get_micro_version()}"

    def assertDeprecation(self, blueprint: str, message: str):
        try:
            tokens = tokenizer.tokenize(blueprint)
            _ast, errors, warnings = parser.parse(tokens)

            self.assertIsNone(errors)
            self.assertEqual(len(warnings), 1)
            self.assertIsInstance(warnings[0], DeprecatedWarning)
            self.assertEqual(warnings[0].message, message)
        except PrintableError as e:  # pragma: no cover
            e.pretty_print("<deprecations test>", blueprint)
            raise AssertionError()

    def test_class_deprecation(self):
        if Gtk.check_version(4, 10, 0) is not None:
            self.skipTest(f"Gtk.Dialog is not deprecated in GTK {self.gtkVersion}")

        blueprint = """
        using Gtk 4.0;

        Dialog {
            use-header-bar: 1;
        }
        """
        message = "Gtk.Dialog is deprecated"

        self.assertDeprecation(blueprint, message)

    def test_property_deprecation(self):
        self.skipTest(
            "gobject-introspection does not currently write property deprecations to the typelib. See <https://gitlab.gnome.org/GNOME/gobject-introspection/-/merge_requests/410>."
        )

        if Gtk.check_version(4, 4, 0) is not None:
            self.skipTest(
                f"Gtk.DropTarget:drop is not deprecated in GTK {self.gtkVersion}"
            )

        blueprint = """
        using Gtk 4.0;

        $MyObject {
            a: bind drop_target.drop;
        }

        DropTarget drop_target {
        }
        """

        message = "Gtk.DropTarget:drop is deprecated"

        self.assertDeprecation(blueprint, message)

    def test_signal_deprecation(self):
        if Gtk.check_version(4, 10, 0) is not None:
            self.skipTest(
                f"Gtk.Window::keys-changed is not deprecated in GTK {self.gtkVersion}"
            )

        blueprint = """
        using Gtk 4.0;

        Window {
            keys-changed => $handler();
        }
        """

        message = "signal Gtk.Window::keys-changed () is deprecated"

        self.assertDeprecation(blueprint, message)
