=======
Objects
=======


.. _Syntax Object:

Objects
-------

.. rst-class:: grammar-block

   Object = :ref:`TypeName<Syntax TypeName>` <id::ref:`IDENT<Syntax IDENT>`>? ObjectContent
   ObjectContent = '{' (:ref:`Signal<Syntax Signal>` | :ref:`Property<Syntax Property>` | :ref:`Extension<Syntax Extension>` | :ref:`Child<Syntax Child>`)* '}'

Objects are the basic building blocks of a GTK user interface. Your widgets are all objects, as are some other features such as list models.

Optionally, objects may have an ID to provide a handle for other parts of the blueprint and your code to access objects.

.. note::

   Object IDs must be unique within their scope. The document root is a scope, but :ref:`sub-templates<Syntax ExtListItemFactory>` have their own, isolated scope.

Example
~~~~~~~

.. code-block:: blueprint

   Label label1 {
     label: "Hello, world!";
   }
   Label label2 {
     label: bind-property file.name;
   }


.. _Syntax TypeName:

Type Names
----------

.. rst-class:: grammar-block

   TypeName = TypeNameFull | TypeNameExternal | TypeNameShort
   TypeNameFull = <namespace::ref:`IDENT<Syntax IDENT>`> '.' <name::ref:`IDENT<Syntax IDENT>`>
   TypeNameExternal = '$' <name::ref:`IDENT<Syntax IDENT>`>
   TypeNameShort = <name::ref:`IDENT<Syntax IDENT>`>

There are three forms of type names: full, short, and external. Full type names take the form ``{namespace}.{name}``, e.g. ``Gtk.ApplicationWindow`` or ``Adw.Leaflet``. Because GTK types are so common, the Gtk namespace may be omitted, shortening ``Gtk.ApplicationWindow`` to just ``ApplicationWindow``.

External type names refer to types defined in your application. They are prefixed with ``$`` and do not have a dot between the namespace and class name. In fact, anywhere a ``$`` is used in a blueprint, it refers to something that must be defined in your application code.


.. _Syntax Property:

Properties
----------

.. rst-class:: grammar-block

   Property = <name::ref:`IDENT<Syntax IDENT>`> ':' ( :ref:`Binding<Syntax Binding>` | :ref:`ObjectValue<Syntax ObjectValue>` | :ref:`Value<Syntax Value>` ) ';'

Properties specify the details of each object, like a label's text, an image's icon name, or the margins on a container.

Most properties are static and directly specified in the blueprint, but properties can also be bound to a data model using the ``bind`` or ``bind-property`` keywords.

A property's value can be another object, either inline or referenced by ID.

Example
~~~~~~~

.. code-block:: blueprint

   Label {
     label: "text";
   }

   Button {
     /* Inline object value. Notice the semicolon after the object. */
     child: Image {
       /* ... */
     };
   }


.. _Syntax Signal:

Signal Handlers
---------------

.. rst-class:: grammar-block

   Signal = <name::ref:`IDENT<Syntax IDENT>`> ('::' <detail::ref:`IDENT<Syntax IDENT>`>)? '=>' '$' <handler::ref:`IDENT<Syntax IDENT>`> '(' <object::ref:`IDENT<Syntax IDENT>`>? ')' (SignalFlag)* ';'
   SignalFlag = 'after' | 'swapped'

Signals are one way to respond to user input (another is `actions <https://docs.gtk.org/gtk4/actions.html>`_, which use the `action-name property <https://docs.gtk.org/gtk4/property.Actionable.action-name.html>`_).

Signals provide a handle for your code to listen to events in the UI. The handler name is prefixed with ``$`` to indicate that it's an external symbol which needs to be provided by your code; if it isn't, things might not work correctly, or at all.

Optionally, you can provide an object ID to use when connecting the signal.

Example
~~~~~~~

.. code-block:: blueprint

   Button {
     clicked => $on_button_clicked();
   }


.. _Syntax Child:

Children
--------

.. rst-class:: grammar-block

   Child = ChildAnnotation? :ref:`Object<Syntax Object>`
   ChildAnnotation = '[' ( ChildInternal | :ref:`ChildExtension<Syntax ChildExtension>` | ChildType ) ']'
   ChildInternal = 'internal-child' <internal-child::ref:`IDENT<Syntax IDENT>`>
   ChildType = <child_type::ref:`IDENT<Syntax IDENT>`>

Some objects can have children. This defines the hierarchical structure of a user interface: containers contain widgets, which can be other containers, and so on.

Child annotations are defined by the parent widget. Some widgets, such as `HeaderBar <https://docs.gtk.org/gtk4/class.HeaderBar.html>`_, have "child types" which allow different child objects to be treated in different ways. Some, such as `Dialog <https://docs.gtk.org/gtk4/class.Dialog.html>`_ and `InfoBar <https://docs.gtk.org/gtk4/class.InfoBar.html>`_, define child :ref:`extensions<Syntax ChildExtension>`, which provide more detailed information about the child.

Internal children are a special case. Rather than creating a new object, children marked with ``[internal-child <name>]`` modify an existing object provided by the parent. This is used, for example, for the ``content_area`` of a `Dialog <https://docs.gtk.org/gtk4/class.Dialog.html>`_.

.. note::

   The objects at the root of a blueprint cannot have child annotations, since there is no root widget for them to be a child of.

.. note::

   Some widgets, like `Button <https://docs.gtk.org/gtk4/class.Button.html>`_, use a property to set their child instead. Widgets added in this way don't have child annotations.

Examples
~~~~~~~~

Add children to a container
+++++++++++++++++++++++++++

.. code-block:: blueprint

   Button {
     Image {}
   }

Child types
+++++++++++

.. code-block:: blueprint

   HeaderBar {
     [start]
     Label {
     }

     [end]
     Button {
     }
   }

Child extensions
++++++++++++++++

.. code-block:: blueprint

   Dialog {
     // Here, a child extension annotation defines the button's response.
     [action response=cancel]
     Button {}
   }

Internal children
+++++++++++++++++

.. code-block:: blueprint

   Dialog {
     [internal-child content_area]
     Box {
       // Unlike most objects in a blueprint, this internal-child widget
       // represents the properties, signal handlers, children, and extensions
       // of an existing Box created by the Dialog, not a new Box created by
       // the blueprint.
     }
   }
