# ast_utils.py
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

import typing as T
from collections import ChainMap, defaultdict
from functools import cached_property

from .errors import *
from .lsp_utils import DocumentSymbol, LocationLink, SemanticToken
from .tokenizer import Range

TType = T.TypeVar("TType")


class Children:
    """Allows accessing children by type using array syntax."""

    def __init__(self, children):
        self._children = children

    def __iter__(self) -> T.Iterator["AstNode"]:
        return iter(self._children)

    @T.overload
    def __getitem__(self, key: T.Type[TType]) -> T.List[TType]: ...

    @T.overload
    def __getitem__(self, key: int) -> "AstNode": ...

    def __getitem__(self, key):
        if isinstance(key, int):
            if key >= len(self._children):
                return None
            else:
                return self._children[key]
        else:
            return [child for child in self._children if isinstance(child, key)]


class Ranges:
    def __init__(self, ranges: T.Dict[str, Range]):
        self._ranges = ranges

    def __getitem__(self, key: T.Union[str, tuple[str, str]]) -> T.Optional[Range]:
        if isinstance(key, str):
            return self._ranges.get(key)
        elif isinstance(key, tuple):
            start, end = key
            return Range.join(self._ranges.get(start), self._ranges.get(end))


TCtx = T.TypeVar("TCtx")
TAttr = T.TypeVar("TAttr")


class Ctx:
    """Allows accessing values from higher in the syntax tree."""

    def __init__(self, node: "AstNode") -> None:
        self.node = node

    def __getitem__(self, key: T.Type[TCtx]) -> T.Optional[TCtx]:
        attrs = self.node._attrs_by_type(Context)
        for name, attr in attrs:
            if attr.type == key:
                return getattr(self.node, name)
        if self.node.parent is not None:
            return self.node.parent.context[key]
        else:
            return None


class AstNode:
    """Base class for nodes in the abstract syntax tree."""

    completers: T.List = []
    attrs_by_type: T.Dict[T.Type, T.List] = {}

    def __init__(self, group, children, tokens, incomplete=False):
        self.group = group
        self.children = Children(children)
        self.tokens = ChainMap(tokens, defaultdict(lambda: None))
        self.incomplete = incomplete

        self.parent = None
        for child in self.children:
            child.parent = self

    def __init_subclass__(cls):
        cls.completers = []
        cls.validators = [
            getattr(cls, f) for f in dir(cls) if hasattr(getattr(cls, f), "_validator")
        ]
        cls.attrs_by_type = {}

    @cached_property
    def context(self):
        return Ctx(self)

    @cached_property
    def ranges(self):
        return Ranges(self.group.ranges)

    @cached_property
    def root(self):
        if self.parent is None:
            return self
        else:
            return self.parent.root

    @property
    def range(self) -> Range:
        return Range(self.group.start, self.group.end, self.group.text)

    def parent_by_type(self, type: T.Type[TType]) -> TType:
        if self.parent is None:
            raise CompilerBugError()
        elif isinstance(self.parent, type):
            return self.parent
        else:
            return self.parent.parent_by_type(type)

    @cached_property
    def errors(self):
        return list(
            error
            for error in self._get_errors()
            if not isinstance(error, CompileWarning)
        )

    @cached_property
    def warnings(self):
        return list(
            warning
            for warning in self._get_errors()
            if isinstance(warning, CompileWarning)
        )

    def _get_errors(self):
        for validator in self.validators:
            try:
                validator(self)
            except CompileError as e:
                yield e
                if e.fatal:
                    return

        for child in self.children:
            yield from child._get_errors()

    def _attrs_by_type(self, attr_type: T.Type[TAttr]) -> T.List[T.Tuple[str, TAttr]]:
        if attr_type not in self.attrs_by_type:
            self.attrs_by_type[attr_type] = []
            for name in dir(type(self)):
                item = getattr(type(self), name)
                if isinstance(item, attr_type):
                    self.attrs_by_type[attr_type].append((name, item))
        return self.attrs_by_type[attr_type]

    def get_docs(self, idx: int) -> T.Optional[str]:
        for name, attr in self._attrs_by_type(Docs):
            if attr.token_name:
                token = self.group.tokens.get(attr.token_name)
                if token and token.start <= idx < token.end:
                    return getattr(self, name)

        for child in self.children:
            if idx in child.range:
                if docs := child.get_docs(idx):
                    return docs

        for name, attr in self._attrs_by_type(Docs):
            if not attr.token_name:
                return getattr(self, name)

        return None

    def get_semantic_tokens(self) -> T.Iterator[SemanticToken]:
        for child in self.children:
            yield from child.get_semantic_tokens()

    def get_reference(self, idx: int) -> T.Optional[LocationLink]:
        for child in self.children:
            if idx in child.range:
                if ref := child.get_reference(idx):
                    return ref
        return None

    @property
    def document_symbol(self) -> T.Optional[DocumentSymbol]:
        return None

    def get_document_symbols(self) -> T.List[DocumentSymbol]:
        result = []
        for child in self.children:
            if s := child.document_symbol:
                s.children = child.get_document_symbols()
                result.append(s)
            else:
                result.extend(child.get_document_symbols())
        return result

    def validate_unique_in_parent(
        self, error: str, check: T.Optional[T.Callable[["AstNode"], bool]] = None
    ):
        for child in self.parent.children:
            if child is self:
                break

            if type(child) is type(self):
                if check is None or check(child):
                    raise CompileError(
                        error,
                        references=[
                            ErrorReference(
                                child.range,
                                "previous declaration was here",
                            )
                        ],
                    )


def validate(
    token_name: T.Optional[str] = None,
    end_token_name: T.Optional[str] = None,
    skip_incomplete: bool = False,
):
    """Decorator for functions that validate an AST node. Exceptions raised
    during validation are marked with range information from the tokens."""

    def decorator(func):
        def inner(self: AstNode):
            if skip_incomplete and self.incomplete:
                return

            try:
                func(self)
            except CompileError as e:
                # If the node is only partially complete, then an error must
                # have already been reported at the parsing stage
                if self.incomplete:
                    return

                if e.range is None:
                    e.range = (
                        Range.join(
                            self.ranges[token_name],
                            self.ranges[end_token_name],
                        )
                        or self.range
                    )

                # Re-raise the exception
                raise e

        inner._validator = True
        return inner

    return decorator


class Docs:
    def __init__(self, func, token_name=None):
        self.func = func
        self.token_name = token_name

    def __get__(self, instance, owner):
        if instance is None:
            return self
        return self.func(instance)


def docs(*args, **kwargs):
    """Decorator for functions that return documentation for tokens."""

    def decorator(func):
        return Docs(func, *args, **kwargs)

    return decorator


class Context:
    def __init__(self, type: T.Type[TCtx], func: T.Callable[[AstNode], TCtx]) -> None:
        self.type = type
        self.func = func

    def __get__(self, instance, owner):
        if instance is None:
            return self
        if ctx := getattr(instance, "_context_" + self.type.__name__, None):
            return ctx
        else:
            ctx = self.func(instance)
            setattr(instance, "_context_" + self.type.__name__, ctx)
            return ctx


def context(type: T.Type[TCtx]):
    """Decorator for functions that return a context object, which is passed down to ."""

    def decorator(func: T.Callable[[AstNode], TCtx]) -> Context:
        return Context(type, func)

    return decorator
