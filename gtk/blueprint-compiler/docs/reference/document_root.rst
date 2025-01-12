=======================
Document Root & Imports
=======================


.. _Syntax Root:

Document Root
-------------

.. rst-class:: grammar-block

   Root = :ref:`GtkDecl<Syntax GtkDecl>` (:ref:`Using<Syntax Using>`)* (:ref:`TranslationDomain<Syntax TranslationDomain>`)? ( :ref:`Template<Syntax Template>` | :ref:`Menu<Syntax Menu>` | :ref:`Object<Syntax Object>` )* EOF

A blueprint document consists of a :ref:`GTK declaration<Syntax GtkDecl>`, one or more :ref:`imports<Syntax Using>`, and a list of :ref:`objects<Syntax Object>` and/or a :ref:`template<Syntax Template>`.

Example
~~~~~~~

.. code-block:: blueprint

   // Gtk Declaration
   using Gtk 4.0;

   // Import Statement
   using Adw 1;

   // Object
   Window my_window {}


.. _Syntax GtkDecl:

GTK Declaration
---------------

.. rst-class:: grammar-block

   GtkDecl = 'using' 'Gtk' '4.0' ';'

Every blueprint file begins with the line ``using Gtk 4.0;``, which declares the target GTK version for the file. Tools that read blueprint files should verify that they support the declared version.

Example
~~~~~~~

.. code-block:: blueprint

   using Gtk 4.0;


.. _Syntax Using:

GObject Introspection Imports
-----------------------------

.. rst-class:: grammar-block

   Using = 'using' <namespace::ref:`IDENT<Syntax IDENT>`> <version::ref:`NUMBER<Syntax NUMBER>`> ';'

To use classes and types from namespaces other than GTK itself, those namespaces must be imported at the top of the file. This tells the compiler what version of the namespace to import.

You'll need the GIR name and version, not the package name and not the exact version number. These are listed at the top of each library's documentation homepage:

.. image:: gir-namespace.png

The compiler requires typelib files for these libraries to be installed. They are usually installed with the library, but on some distros, you may need to install the package that provides ``{namespace}-{version}.typelib`` (e.g. ``Adw-1.typelib``).

Example
~~~~~~~

.. code-block:: blueprint

   // Import libadwaita
   using Adw 1;


.. _Syntax TranslationDomain:

Translation Domain
------------------

.. rst-class:: grammar-block

   TranslationDomain = 'translation-domain' <domain::ref:`QUOTED<Syntax QUOTED>`> ';'

The translation domain is used to look up translations for translatable strings in the blueprint file. If no translation domain is specified, strings will be looked up in the program's global domain.

See `Gtk.Builder:translation-domain <https://docs.gtk.org/gtk4/property.Builder.translation-domain.html>`_ for more information.
