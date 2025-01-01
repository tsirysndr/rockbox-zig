================
Syntax Reference
================

This is the official specification of the blueprint format.

The grammar is expressed as a `parsing expression grammar <https://en.wikipedia.org/wiki/Parsing_expression_grammar>`_. This has two important implications: the parser will never backtrack, and alternation (e.g. a ``|`` in the specification) will always take the *first* branch that matches, even if that causes an error later. These properties make PEGs both unambiguous and simple to implement in code.

Blueprint uses C-style line comments (``// comment for the rest of the line``) and block comments (``/* multiline comment... */``).

Wherever commas are used as delimiters in repetition (expressed in this reference as ``( <rule> ),*``), the trailing comma is permitted and optional.

.. toctree::
   :maxdepth: 1

   document_root
   objects
   templates
   values
   expressions
   menus
   extensions
   diagnostics


Tokens
------

.. _Syntax IDENT:

IDENT
~~~~~

An identifier starts with an ASCII underscore ``_`` or letter ``[A-Za-z]`` and consists of ASCII underscores, letters, digits ``[0-9]``, and dashes ``-``. Dashes are included for historical reasons, since GObject properties and signals are traditionally kebab-case.

.. _Syntax NUMBER:

NUMBER
~~~~~~

Numbers begin with an ASCII digit and consist of ASCII digits, underscores, dots ``.``, and letters (for radix pre-/suffixes). More than one dot in a number is not allowed. Underscores are permitted for increased readability, and are ignored.

Hexadecimal numbers may be specified using the ``0x`` prefix and may use uppercase or lowercase letters, or a mix. Hexadecimal values may not have a fractional part. They are generally converted to decimal in the output.

.. _Syntax QUOTED:

QUOTED
~~~~~~

Quotes begin with an ASCII single quote ``'`` or double quote ``"`` and end with the same character they started with. An ASCII backslash ``\`` begins an escape sequence; this allows newlines ``\n``, tabs ``\t``, and quotes ``\'``, ``\"`` to be inserted. It also allows multiline strings by escaping a newline character, which will be ignored.
