===================
Composite Templates
===================

.. _Syntax Template:

Composite Templates
-------------------

.. rst-class:: grammar-block

   Template = 'template' :ref:`TypeName<Syntax TypeName>` ( ':' :ref:`TypeName<Syntax TypeName>` )? :ref:`ObjectContent<Syntax Object>`

Widget subclassing is one of the primary techniques for structuring an application. For example, a maps app might have a `Gtk.ApplicationWindow <https://docs.gtk.org/gtk4/class.ApplicationWindow.html>`_ subclass, ``MapsApplicationWindow``, that implements the functionality of its main window. But a maps app has a lot of functionality, so the headerbar might be split into its own `Gtk.HeaderBar <https://docs.gtk.org/gtk4/class.HeaderBar.html>`_ subclass, ``MapsHeaderBar``, for the sake of organization.

You could implement this with the following blueprint:

.. code-block:: blueprint

   using Gtk 4.0;

   $MapsApplicationWindow window {
     $MapsHeaderBar {
       /* probably a lot of buttons ... */
     }

     $MapsMainView {
       /* a lot more UI definitions ... */
     }
   }

There are two problems with this approach:

1. The widget code may be organized neatly into different files, but the UI is not. This blueprint contains the entire UI definition for the app.

2. Widgets aren't in control of their own contents. It shouldn't be up to the caller to construct a widget using the correct blueprint--that's an implementation detail of the widget.

We can solve these problems by giving each widget its own blueprint file, which we reference in the widget's constructor. Then, whenever the widget is instantiated (by another blueprint, or by the application), it will get all the children and properties defined in its blueprint.

For this to work, we need to specify in the blueprint which object is the one being instantiated. We do this with a template block:

.. code-block:: blueprint

   using Gtk 4.0;

   template $MapsHeaderBar : Gtk.HeaderBar {
     /* probably a lot of buttons ... */
   }

   Gio.ListStore bookmarked_places_store {
     /* This isn't the object being instantiated, just an auxillary object. GTK knows this because it isn't the
        one marked with 'template'. */
   }

This blueprint can only be used by the ``MapsHeaderBar`` constructor. Instantiating it with ``Gtk.Builder`` won't work since it needs an existing, under-construction ``MapsHeaderBar`` to use for the template object. The ``template`` block must be at the top level of the file (not nested within another object) and only one is allowed per file.

This ``MapsHeaderBar`` class, along with its blueprint template, can then be referenced in another blueprint:

.. code-block:: blueprint

   using Gtk 4.0;

   ApplicationWindow {
     $MapsHeaderBar {
       /* Nothing needed here, the widgets are in the MapsHeaderBar template. */
     }
   }

Type & Parent Parameters
~~~~~~~~~~~~~~~~~~~~~~~~

The type name that directly follows the ``template`` keyword is the type of the template class. In most cases, this will be an extern type starting with ``$`` and matching the class name in the application code. Templates for use in a `Gtk.BuilderListItemFactory <https://docs.gtk.org/gtk4/class.BuilderListItemFactory.html>`_ use ``ListItem`` as the type name instead.

The parent type is optional, and may only be present if the template type is extern. It enables limited type checking for the properties and signals of the template object.


Referencing a Template
----------------------

To reference the template object in a binding or expression, use the ``template`` keyword:

.. code-block:: blueprint

   template $MyTemplate {
     prop1: "Hello, world!";
     prop2: bind template.prop1;
   }


Language Implementations
------------------------

- **C** ``gtk_widget_class_set_template ()``: https://docs.gtk.org/gtk4/class.Widget.html#building-composite-widgets-from-template-xml
- **gtk-rs** ``#[template]``: https://gtk-rs.org/gtk4-rs/stable/latest/book/composite_templates.html
- **GJS** ``GObject.registerClass()``: https://gjs.guide/guides/gtk/3/14-templates.html
- **PyGObject** ``@Gtk.Template``: https://pygobject.gnome.org/guide/gtk_template.html
