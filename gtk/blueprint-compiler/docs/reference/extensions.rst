==========
Extensions
==========

.. _Syntax Extension:

Properties are the main way to set values on objects, but they are limited by the GObject type system in what values they can accept. Some classes, therefore, have specialized syntax for certain features.

.. note::

   Extensions are a feature of ``Gtk.Buildable``--see `Gtk.Buildable.custom_tag_start() <https://docs.gtk.org/gtk4/vfunc.Buildable.custom_tag_start.html>`_ for internal details.

   Because they aren't part of the type system, they aren't present in typelib files like properties and signals are. Therefore, if a library adds a new extension, syntax for it must be added to Blueprint manually. If there's a commonly used extension that isn't supported by Blueprint, please `file an issue <https://gitlab.gnome.org/jwestman/blueprint-compiler/-/issues>`_.

.. rst-class:: grammar-block

   Extension = :ref:`ExtAccessibility<Syntax ExtAccessibility>`
   | :ref:`ExtAdwAlertDialog<Syntax ExtAdwAlertDialog>`
   | :ref:`ExtAdwMessageDialog<Syntax ExtAdwMessageDialog>`
   | :ref:`ExtAdwBreakpoint<Syntax ExtAdwBreakpoint>`
   | :ref:`ExtComboBoxItems<Syntax ExtComboBoxItems>`
   | :ref:`ExtFileFilterMimeTypes<Syntax ExtFileFilter>`
   | :ref:`ExtFileFilterPatterns<Syntax ExtFileFilter>`
   | :ref:`ExtFileFilterSuffixes<Syntax ExtFileFilter>`
   | :ref:`ExtLayout<Syntax ExtLayout>`
   | :ref:`ExtListItemFactory<Syntax ExtListItemFactory>`
   | :ref:`ExtSizeGroupWidgets<Syntax ExtSizeGroupWidgets>`
   | :ref:`ExtStringListStrings<Syntax ExtStringListStrings>`
   | :ref:`ExtStyles<Syntax ExtStyles>`


.. _Syntax ExtAccessibility:

Accessibility Properties
------------------------

.. rst-class:: grammar-block

   ExtAccessibility = 'accessibility' '{' ExtAccessibilityProp* '}'
   ExtAccessibilityProp = <name::ref:`IDENT<Syntax IDENT>`> ':' (:ref:`Value <Syntax Value>` | ('[' (:ref: Value <Syntax Value>),* ']') ) ';'

Valid in any `Gtk.Widget <https://docs.gtk.org/gtk4/class.Widget.html>`_.

The ``accessibility`` block defines values relevant to accessibility software. The property names and acceptable values are described in the `Gtk.AccessibleRelation <https://docs.gtk.org/gtk4/enum.AccessibleRelation.html>`_, `Gtk.AccessibleState <https://docs.gtk.org/gtk4/enum.AccessibleState.html>`_, and `Gtk.AccessibleProperty <https://docs.gtk.org/gtk4/enum.AccessibleProperty.html>`_ enums.

.. note::

   Relations which allow for a list of values, for example `labelled-by`, must be given as a single relation with a list of values instead of duplicating the relation like done in Gtk.Builder.

.. _Syntax ExtAdwBreakpoint:

Adw.Breakpoint
--------------

.. rst-class:: grammar-block

   ExtAdwBreakpointCondition = 'condition' '(' <condition::ref:`QUOTED<Syntax QUOTED>`> ')'
   ExtAdwBreakpoint = 'setters' '{' ExtAdwBreakpointSetter* '}'
   ExtAdwBreakpointSetter = <object::ref:`IDENT<Syntax IDENT>`> '.' <property::ref:`IDENT<Syntax IDENT>`> ':' :ref:`Value <Syntax Value>` ';'

Valid in `Adw.Breakpoint <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Breakpoint.html>`_.

Defines the condition for a breakpoint and the properties that will be set at that breakpoint. See the documentation for `Adw.Breakpoint <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Breakpoint.html>`_.

.. note::

   The `Adw.Breakpoint:condition <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/property.Breakpoint.condition.html>`_ property has type `Adw.BreakpointCondition <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/struct.BreakpointCondition.html>`_, which GtkBuilder doesn't know how to parse from a string. Therefore, the ``condition`` syntax is used instead.


.. _Syntax ExtAdwAlertDialog:

Adw.AlertDialog Responses
----------------------------

.. rst-class:: grammar-block

   ExtAdwAlertDialog = 'responses' '[' (ExtAdwAlertDialogResponse),* ']'
   ExtAdwAlertDialogResponse = <id::ref:`IDENT<Syntax IDENT>`> ':' :ref:`StringValue<Syntax StringValue>` ExtAdwAlertDialogFlag*
   ExtAdwAlertDialogFlag = 'destructive' | 'suggested' | 'disabled'

Valid in `Adw.AlertDialog <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1-latest/class.AlertDialog.html>`_.

The ``responses`` block defines the buttons that will be added to the dialog. The ``destructive`` or ``suggested`` flag sets the appearance of the button, and the ``disabled`` flag can be used to disable the button.

.. code-block:: blueprint

   using Adw 1;

   Adw.AlertDialog {
     responses [
       cancel: _("Cancel"),
       delete: _("Delete") destructive,
       save: "Save" suggested,
       wipeHardDrive: "Wipe Hard Drive" destructive disabled,
     ]
   }


.. _Syntax ExtAdwMessageDialog:

Adw.MessageDialog Responses
----------------------------

.. rst-class:: grammar-block

   ExtAdwMessageDialog = 'responses' '[' (ExtAdwMessageDialogResponse),* ']'
   ExtAdwMessageDialogResponse = <id::ref:`IDENT<Syntax IDENT>`> ':' :ref:`StringValue<Syntax StringValue>` ExtAdwMessageDialogFlag*
   ExtAdwMessageDialogFlag = 'destructive' | 'suggested' | 'disabled'

Valid in `Adw.MessageDialog <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1-latest/class.MessageDialog.html>`_.

The ``responses`` block defines the buttons that will be added to the dialog. The ``destructive`` or ``suggested`` flag sets the appearance of the button, and the ``disabled`` flag can be used to disable the button.

.. code-block:: blueprint

   using Adw 1;

   Adw.MessageDialog {
     responses [
       cancel: _("Cancel"),
       delete: _("Delete") destructive,
       save: "Save" suggested,
       wipeHardDrive: "Wipe Hard Drive" destructive disabled,
     ]
   }


.. _Syntax ExtComboBoxItems:

Gtk.ComboBoxText Items
----------------------

.. rst-class:: grammar-block

   ExtComboBoxItems = 'items' '[' (ExtComboBoxItem),* ']'
   ExtComboBoxItem = ( <id::ref:`IDENT<Syntax IDENT>`> ':' )? :ref:`StringValue<Syntax StringValue>`

Valid in `Gtk.ComboBoxText <https://docs.gtk.org/gtk4/class.ComboBoxText.html>`_, which is deprecated as of Gtk 4.10.

The ``items`` block defines the items that will be added to the combo box. The optional ID can be used to refer to the item rather than its label.

.. code-block:: blueprint

   ComboBoxText {
     items [
       item1: "Item 1",
       item2: "Item 2",
       item3: "Item 3",
     ]
   }


.. _Syntax ExtFileFilter:

Gtk.FileFilter Filters
----------------------

.. rst-class:: grammar-block

   ExtFileFilterMimeTypes = 'mime-types' '[' (ExtFileFilterItem),* ']'
   ExtFileFilterPatterns = 'patterns' '[' (ExtFileFilterItem),* ']'
   ExtFileFilterSuffixes = 'suffixes' '[' (ExtFileFilterItem),* ']'
   ExtFileFilterItem = <item::ref:`QUOTED<Syntax QUOTED>`>

Valid in `Gtk.FileFilter <https://docs.gtk.org/gtk4/class.FileFilter.html>`_.

The ``mime-types``, ``patterns``, and ``suffixes`` blocks define the items that will be added to the file filter. The ``mime-types`` block accepts mime types (including wildcards for subtypes, such as ``image/*``). The ``patterns`` block accepts glob patterns, and the ``suffixes`` block accepts file extensions.

.. code-block:: blueprint

   FileFilter {
     mime-types [ "text/plain", "image/*" ]
     patterns [ "*.txt" ]
     suffixes [ "png", "jpg" ]
   }


.. _Syntax ExtLayout:

Widget Layouts
--------------

.. rst-class:: grammar-block

   ExtLayout = 'layout' '{' ExtLayoutProp* '}'
   ExtLayoutProp = <name::ref:`IDENT<Syntax IDENT>`> ':' :ref:`Value<Syntax Value>` ';'

Valid in `Gtk.Widget <https://docs.gtk.org/gtk4/class.Widget.html>`_.

The ``layout`` block describes how the widget should be positioned within its parent. The available properties depend on the parent widget's layout manager.

.. code-block:: blueprint

   Grid {
     Button {
       layout {
         column: 0;
         row: 0;
       }
     }
     Button {
       layout {
         column: 1;
         row: 0;
       }
     }
     Button {
       layout {
         column: 0;
         row: 1;
         row-span: 2;
       }
     }
   }


.. _Syntax ExtListItemFactory:

Gtk.BuilderListItemFactory Templates
------------------------------------

.. rst-class:: grammar-block

   ExtListItemFactory = 'template' :ref:`TypeName<Syntax TypeName>` :ref:`ObjectContent<Syntax Object>`

Valid in `Gtk.BuilderListItemFactory <https://docs.gtk.org/gtk4/class.BuilderListItemFactory.html>`_.

The ``template`` block defines the template that will be used to create list items. This block is unique within Blueprint because it defines a completely separate sub-blueprint which is used to create each list item. The sub-blueprint may not reference objects in the main blueprint or vice versa.

The template type must be `Gtk.ListItem <https://docs.gtk.org/gtk4/class.ListItem.html>`_, `Gtk.ColumnViewRow <https://docs.gtk.org/gtk4/class.ColumnViewRow.html>`_, or `Gtk.ColumnViewCell <https://docs.gtk.org/gtk4/class.ColumnViewCell.html>`_. The template object can be referenced with the ``template`` keyword.

.. code-block:: blueprint

   ListView {
     factory: BuilderListItemFactory {
       template ListItem {
         child: Label {
           label: bind template.item as <StringObject>.string;
         };
       }
     };

     model: NoSelection {
       model: StringList {
         strings [ "Item 1", "Item 2", "Item 3" ]
       };
     };
   }


.. _Syntax ExtScaleMarks:

Gtk.Scale Marks
---------------

.. rst-class:: grammar-block

   ExtScaleMarks = 'marks' '[' (ExtScaleMark),* ']'
   ExtScaleMark = 'mark' '(' ( '-' | '+' )? <value::ref:`NUMBER<Syntax NUMBER>`> ( ',' <position::ref:`IDENT<Syntax IDENT>`> ( ',' :ref:`StringValue<Syntax StringValue>` )? )? ')'

Valid in `Gtk.Scale <https://docs.gtk.org/gtk4/class.Scale.html>`_.

The ``marks`` block defines the marks on a scale. A single ``mark`` has up to three arguments: a value, an optional position, and an optional label. The position can be ``left``, ``right``, ``top``, or ``bottom``. The label may be translated.


.. _Syntax ExtSizeGroupWidgets:

Gtk.SizeGroup Widgets
---------------------

.. rst-class:: grammar-block

   ExtSizeGroupWidgets = 'widgets' '[' (ExtSizeGroupWidget),* ']'
   ExtSizeGroupWidget = <id::ref:`IDENT<Syntax IDENT>`>

Valid in `Gtk.SizeGroup <https://docs.gtk.org/gtk4/class.SizeGroup.html>`_.

The ``widgets`` block defines the widgets that will be added to the size group.

.. code-block:: blueprint

   Box {
     Button button1 {}
     Button button2 {}
   }

   SizeGroup {
     widgets [button1, button2]
   }


.. _Syntax ExtStringListStrings:

Gtk.StringList Strings
----------------------

.. rst-class:: grammar-block

   ExtStringListStrings = 'strings' '[' (ExtStringListItem),* ']'
   ExtStringListItem = :ref:`StringValue<Syntax StringValue>`

Valid in `Gtk.StringList <https://docs.gtk.org/gtk4/class.StringList.html>`_.

The ``strings`` block defines the strings in the string list.

.. code-block:: blueprint

   StringList {
     strings ["violin", "guitar", _("harp")]
   }


.. _Syntax ExtStyles:

CSS Styles
----------

.. rst-class:: grammar-block

   ExtStyles = 'styles' '[' ExtStylesProp* ']'
   ExtStylesProp = <name::ref:`QUOTED<Syntax QUOTED>`>

Valid in any `Gtk.Widget <https://docs.gtk.org/gtk4/class.Widget.html>`_.

The ``styles`` block defines CSS classes that will be added to the widget.

.. code-block:: blueprint

   Button {
     styles ["suggested-action"]
   }


.. _Syntax ChildExtension:

Child Extensions
----------------

.. rst-class:: grammar-block

   ChildExtension = :ref:`ExtResponse<Syntax ExtResponse>`

Child extensions are similar to regular extensions, but they apply to a child of the object rather than the object itself. They are used to add properties to child widgets of a container, such as the buttons in a `Gtk.Dialog <https://docs.gtk.org/gtk4/class.Dialog.html>`_. The child extension takes the place of a child type inside the square brackets.

Currently, the only child extension is :ref:`ExtResponse<Syntax ExtResponse>`.


.. _Syntax ExtResponse:

Dialog & InfoBar Responses
--------------------------

.. rst-class:: grammar-block

   ExtResponse = 'action' 'response' '=' ( <name::ref:`IDENT<Syntax IDENT>`> | <id::ref:`NUMBER<Syntax NUMBER>`> ) 'default'?

Valid as a child extension for children of `Gtk.Dialog <https://docs.gtk.org/gtk4/class.Dialog.html>`_ or `Gtk.InfoBar <https://docs.gtk.org/gtk4/class.InfoBar.html>`_, which are both deprecated as of Gtk 4.10.

The ``action response`` extension sets the ``action`` child type for the child and sets the child's integer response type. The response type may be either a member of the `Gtk.ResponseType <https://docs.gtk.org/gtk4/enum.ResponseType.html>`_ enum or a positive, application-defined integer.

No more than one child of a dialog or infobar may have the ``default`` flag.

.. code-block:: blueprint

   Dialog {
    [action response=ok default]
    Button {}

    [action response=cancel]
    Button {}

    [action response=1]
    Button {}
   }
