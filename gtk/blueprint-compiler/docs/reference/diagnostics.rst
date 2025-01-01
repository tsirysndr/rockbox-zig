===========
Diagnostics
===========


.. _Diagnostic abstract_class:

abstract_class
--------------
Objects can't be created from abstract classes. Abstract classes are used as base classes for other classes, but they don't have functionality on their own. You may want to use a non-abstract subclass instead.


.. _Diagnostic bad_syntax:

bad_syntax
----------
The tokenizer encountered an unexpected sequence of characters that aren't part of any known blueprint syntax.


.. _Diagnostic child_not_accepted:

child_not_accepted
------------------
The parent class does not have child objects (it does not implement `Gtk.Buildable <https://docs.gtk.org/gtk4/iface.Buildable.html>`_ and is not a subclass of `Gio.ListStore <https://docs.gtk.org/gio/class.ListStore.html>`_). Some classes use properties instead of children to add widgets. Check the parent class's documentation.


.. _Diagnostic conversion_error:

conversion_error
----------------
The value's type cannot be converted to the target type.

Subclasses may be converted to their superclasses, but not vice versa. A type that implements an interface can be converted to that interface's type. Many boxed types can be parsed from strings in a type-specific way.


.. _Diagnostic expected_bool:

expected_bool
-------------
A boolean value was expected, but the value is not ``true`` or ``false``.


.. _Diagnostic extension_not_repeatable:

extension_not_repeatable
------------------------
This extension can't be used more than once in an object.


.. _Diagnostic extension_wrong_parent_type:

extension_wrong_parent_type
---------------------------
No extension with the given name exists for this object's class (or, for a :ref:`child extension<Syntax ChildExtension>`, the parent class).


.. _Diagnostic invalid_number_literal:

invalid_number_literal
----------------------
The tokenizer encountered what it thought was a number, but it couldn't parse it as a number.


.. _Diagnostic member_dne:

member_dne
----------
The value is being interpreted as a member of an enum or flag type, but that type doesn't have a member with the given name.


.. _Diagnostic missing_gtk_declaration:

missing_gtk_declaration
-----------------------
All blueprint files must start with a GTK declaration, e.g. ``using Gtk 4.0;``.


.. _Diagnostic multiple_templates:

multiple_templates
------------------
Only one :ref:`template<Syntax Template>` is allowed per blueprint file, but there are multiple. The template keyword indicates which object is the one being instantiated.


.. _Diagnostic namespace_not_found:

namespace_not_found
--------------------
The ``.typelib`` files for the given namespace could not be found. There are several possibilities:

* There is a typo in the namespace name, e.g. ``Adwaita`` instead of ``Adw``

* The version number is incorrect, e.g. ``Adw 1.0`` instead of ``Adw 1``. The library's documentation will tell you the correct version number to use.

* The packages for the library are not installed. On some distributions, the ``.typelib`` file is in a separate package from the main library, such as a ``-devel`` package.

* There is an issue with the path to the typelib file. The ``GI_TYPELIB_PATH`` environment variable can be used to add additional paths to search.


.. _Diagnostic namespace_not_imported:

namespace_not_imported
----------------------
The given namespace was not imported at the top of the file. Importing the namespace is necessary because it tells blueprint-compiler which version of the library to use.


.. _Diagnostic object_dne:

object_dne
----------
No object with the given ID exists in the current scope.


.. _Diagnostic property_dne:

property_dne
------------
The class or interface doesn't have a property with the given name.


.. _Diagnostic property_convert_error:

property_convert_error
----------------------
The value given for the property can't be converted to the property's type.


.. _Diagnostic property_construct_only:

property_construct_only
-----------------------
The property can't be bound because it is a construct-only property, meaning it can only be set once when the object is first constructed. Binding it to an expression could cause its value to change later.


.. _Diagnostic property_read_only:

property_read_only
------------------
This property can't be set because it is marked as read-only.


.. _Diagnostic signal_dne:

signal_dne
----------
The class or interface doesn't have a signal with the given name.


.. _Diagnostic type_dne:

type_dne
--------
The given type doesn't exist in the namespace.


.. _Diagnostic type_not_a_class:

type_not_a_class
----------------
The given type exists in the namespace, but it isn't a class. An object's type must be a concrete (not abstract) class, not an interface or boxed type.


.. _Diagnostic version_conflict:

version_conflict
----------------
This error occurs when two versions of a namespace are imported (possibly transitively) in the same file. For example, this will cause a version conflict:

.. code-block:: blueprint

   using Gtk 4.0;
   using Gtk 3.0;

But so will this:

.. code-block:: blueprint

   using Gtk 4.0;
   using Handy 1;

because libhandy imports ``Gtk 3.0``.


.. _Diagnostic wrong_compiler_version:

wrong_compiler_version
----------------------
This version of blueprint-compiler is for GTK 4 blueprints only. Future GTK versions will use different versions of blueprint-compiler.
