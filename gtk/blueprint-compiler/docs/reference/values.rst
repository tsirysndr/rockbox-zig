======
Values
======


.. _Syntax Value:

Values
------

.. rst-class:: grammar-block

   Value = :ref:`Translated<Syntax Translated>` | :ref:`Flags<Syntax Flags>` | :ref:`Literal<Syntax Literal>`


.. _Syntax Literal:

Literals
--------

.. rst-class:: grammar-block

   Literal = :ref:`TypeLiteral<Syntax TypeLiteral>` | QuotedLiteral | NumberLiteral | IdentLiteral
   QuotedLiteral = <value::ref:`QUOTED<Syntax QUOTED>`>
   NumberLiteral = ( '-' | '+' )? <value::ref:`NUMBER<Syntax NUMBER>`>
   IdentLiteral = <ident::ref:`IDENT<Syntax IDENT>`>

Literals are used to specify values for properties. They can be strings, numbers, references to objects, ``null``, types, boolean values, or enum members.

.. _Syntax TypeLiteral:

Type Literals
-------------

.. rst-class:: grammar-block

   TypeLiteral = 'typeof' '<' :ref:`TypeName<Syntax TypeName>` '>'

Sometimes, you need to specify a type as a value. For example, when creating a list store, you may need to specify the type of the items in the list store. This is done using a ``typeof<>`` literal.

The type of a ``typeof<>`` literal is `GType <https://docs.gtk.org/gobject/alias.Type.html>`_, GObject's "meta-type" for type information.


Example
~~~~~~~

.. code-block:: blueprint

   Gio.ListStore {
     item-type: typeof<GObject.Object>;
   }


.. _Syntax Flags:

Flags
-----

.. rst-class:: grammar-block

   Flags = <first::ref:`IDENT<Syntax IDENT>`> '|' ( <rest::ref:`IDENT<Syntax IDENT>`> )|+

Flags are used to specify a set of options. One or more of the available flag values may be specified, and they are combined using ``|``.

Example
~~~~~~~

.. code-block:: blueprint

   Adw.TabView {
     shortcuts: control_tab | control_shift_tab;
   }


.. _Syntax Translated:

Translated Strings
------------------

.. rst-class:: grammar-block

   Translated = ( '_' '(' <string::ref:`QUOTED<Syntax QUOTED>`> ')' ) | ( '\C_' '(' <context::ref:`QUOTED<Syntax QUOTED>`> ',' <string::ref:`QUOTED<Syntax QUOTED>`> ')' )


Use ``_("...")`` to mark strings as translatable. You can put a comment for translators on the line above if needed.

.. code-block:: blueprint

   Gtk.Label label {
     /* Translators: This is the main text of the welcome screen */
     label: _("Hello, world!");
   }

Use ``C_("context", "...")`` to add a *message context* to a string to disambiguate it, in case the same string appears in different places. Remember, two strings might be the same in one language but different in another depending on context.

.. code-block:: blueprint

   Gtk.Label label {
     /* Translators: This is a section in the preferences window */
     label: C_("preferences window", "Hello, world!");
   }


.. _Syntax Binding:

Bindings
--------

.. rst-class:: grammar-block

   Binding = 'bind' :ref:`Expression<Syntax Expression>` (BindingFlag)*
   BindingFlag = 'inverted' | 'bidirectional' | 'no-sync-create'

Bindings keep a property updated as other properties change. They can be used to keep the UI in sync with application data, or to connect two parts of the UI.

The simplest bindings connect to a property of another object in the blueprint. When that other property changes, the bound property updates as well. More advanced bindings can do multi-step property lookups and can even call application code to compute values. See :ref:`the expressions page<Syntax Expression>`.

Simple Bindings
~~~~~~~~~~~~~~~

A binding that consists of a source object and a single lookup is called a "simple binding". These are implemented using `GObject property bindings <https://docs.gtk.org/gobject/method.Object.bind_property.html>`_ and support a few flags:

- ``inverted``: For boolean properties, the target is set to the inverse of the source property.
- ``bidirectional``: The binding is two-way, so changes to the target property will also update the source property.
- ``no-sync-create``: Normally, when a binding is created, the target property is immediately updated with the current value of the source property. This flag disables that behavior, and the bound property will be updated the next time the source property changes.

Complex Bindings
~~~~~~~~~~~~~~~~

Bindings with more complex expressions are implemented with `Gtk.Expression <https://docs.gtk.org/gtk4/class.Expression.html>`_. These bindings do not support flags.

Example
~~~~~~~

.. code-block:: blueprint

   /* Use bindings to show a label when a switch
    * is active, without any application code */

   Switch show_label {}

   Label {
     visible: bind show_label.active;
     label: _("I'm a label that's only visible when the switch is enabled!");
   }

.. _Syntax ObjectValue:

Object Values
-------------

.. rst-class:: grammar-block

   ObjectValue = :ref:`Object<Syntax Object>`

The value of a property can be an object, specified inline. This is particularly useful for widgets that use a ``child`` property rather than a list of child widgets. Objects constructed in this way can even have IDs and be referenced in other places in the blueprint.

Such objects cannot have child annotations because they aren't, as far as blueprint is concerned, children of another object.


.. _Syntax StringValue:

String Values
-------------

.. rst-class:: grammar-block

   StringValue = :ref:`Translated<Syntax Translated>` | :ref:`QuotedLiteral<Syntax Literal>`

Menus, as well as some :ref:`extensions<Syntax Extension>`, have properties that can only be string literals or translated strings.

.. _Syntax ArrayValue:

Array Values
-------------

.. rst-class:: grammar-block

    ArrayValue = '[' (:ref:`StringValue<Syntax StringValue>`),* ']'

For now, it only supports :ref:`Strings<Syntax StringValue>`. This is because Gtk.Builder only supports string arrays.
