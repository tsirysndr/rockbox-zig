# response_id.py
#
# Copyright 2022 Gleb Smirnov <glebsmirnov0708@gmail.com>
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


import typing as T

from .common import *


class ExtResponse(AstNode):
    """Response ID of action widget."""

    ALLOWED_PARENTS: T.List[T.Tuple[str, str]] = [("Gtk", "Dialog"), ("Gtk", "InfoBar")]

    grammar = [
        Keyword("action"),
        Keyword("response"),
        "=",
        AnyOf(
            UseIdent("response_id"),
            [
                Optional(UseExact("sign", "-")),
                UseNumber("response_id"),
            ],
        ),
        Optional([Keyword("default"), UseLiteral("is_default", True)]),
    ]

    @validate()
    def parent_has_action_widgets(self) -> None:
        """Chech that parent widget has allowed type."""
        from .gobject_object import Object

        container_type = self.parent_by_type(Object).gir_class
        if container_type is None:
            return

        gir = self.root.gir

        for namespace, name in ExtResponse.ALLOWED_PARENTS:
            parent_type = gir.get_type(name, namespace)
            if container_type.assignable_to(parent_type):
                break
        else:
            raise CompileError(
                f"{container_type.full_name} doesn't have action widgets"
            )

    @validate()
    def widget_have_id(self) -> None:
        """Check that action widget have ID."""
        from .gtkbuilder_child import Child

        object = self.parent_by_type(Child).object
        if object.id is None:
            raise CompileError(f"Action widget must have ID")

    @validate("response_id")
    def correct_response_type(self) -> None:
        """Validate response type.

        Response type might be GtkResponseType member
        or positive number.
        """
        gir = self.root.gir
        response = self.tokens["response_id"]

        if self.tokens["sign"] == "-":
            raise CompileError("Numeric response type can't be negative")

        if isinstance(response, float):
            raise CompileError(
                "Response type must be GtkResponseType member or integer," " not float"
            )
        elif not isinstance(response, int):
            responses = gir.get_type("ResponseType", "Gtk").members.keys()
            if response not in responses:
                raise CompileError(f'Response type "{response}" doesn\'t exist')

    @validate("default")
    def no_multiple_default(self) -> None:
        """Only one action widget in dialog can be default."""
        from .gobject_object import Object

        if not self.is_default:
            return

        action_widgets = self.parent_by_type(Object).action_widgets
        for widget in action_widgets:
            if widget == self:
                break
            if widget.tokens["is_default"]:
                raise CompileError("Default response is already set")

    @property
    def response_id(self) -> str:
        return self.tokens["response_id"]

    @property
    def is_default(self) -> bool:
        return self.tokens["is_default"] or False

    @property
    def widget_id(self) -> str:
        """Get action widget ID."""
        from .gtkbuilder_child import Child

        object = self.parent_by_type(Child).object
        return object.id

    @docs()
    def ref_docs(self):
        return get_docs_section("Syntax ExtResponse")

    @docs("response_id")
    def response_id_docs(self):
        if enum := self.root.gir.get_type("ResponseType", "Gtk"):
            if member := enum.members.get(self.response_id, None):
                return member.doc


def decompile_response_type(parent_element, child_element):
    obj_id = None
    for obj in child_element.children:
        if obj.tag == "object":
            obj_id = obj["id"]
            break

    if obj_id is None:
        return None

    for child in parent_element.children:
        if child.tag == "action-widgets":
            for action_widget in child.children:
                if action_widget.cdata == obj_id:
                    response_id = action_widget["response"]
                    is_default = (
                        " default" if decompile.truthy(action_widget["default"]) else ""
                    )
                    return f"[action response={response_id}{is_default}]"

    return None


@decompiler("action-widgets", skip_children=True)
def decompile_action_widgets(ctx, gir):
    # This is handled in the <child> decompiler and decompile_response_type above
    pass
