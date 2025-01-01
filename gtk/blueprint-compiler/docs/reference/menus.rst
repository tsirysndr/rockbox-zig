=====
Menus
=====

.. _Syntax Menu:

Menus
-----

.. rst-class:: grammar-block

   Menu = 'menu' <id::ref:`IDENT<Syntax IDENT>`>? '{' MenuChild* '}'
   MenuChild = ( MenuSection | MenuSubmenu | :ref:`MenuItemShorthand<Syntax MenuItemShorthand>` | MenuItem )
   MenuSection = 'section' <id::ref:`IDENT<Syntax IDENT>`>? '{' ( MenuChild | MenuAttribute )* '}'
   MenuSubmenu = 'submenu' <id::ref:`IDENT<Syntax IDENT>`>? '{' ( MenuChild | MenuAttribute )* '}'
   MenuItem = 'item' '{' MenuAttribute* '}'
   MenuAttribute = <name::ref:`IDENT<Syntax IDENT>`> ':' :ref:`StringValue<Syntax StringValue>` ';'

Menus, such as the application menu, are defined using the ``menu`` keyword. Menus have the type `Gio.MenuModel <https://docs.gtk.org/gio/class.MenuModel.html>`_ and can be referenced by ID. They cannot be defined inline.

Example
~~~~~~~

.. code-block:: blueprint

   menu my_menu {
     submenu {
       label: _("File");
       item {
         label: _("New");
         action: "app.new";
         icon: "document-new-symbolic";
       }
     }
   }

   MenuButton {
     menu-model: my_menu;
   }


.. _Syntax MenuItemShorthand:

Item Shorthand
--------------

.. rst-class:: grammar-block

   MenuItemShorthand = 'item' '(' :ref:`StringValue<Syntax StringValue>` ( ',' ( :ref:`StringValue<Syntax StringValue>` ( ',' :ref:`StringValue<Syntax StringValue>`? )? )? )? ')'

The most common menu attributes are ``label``, ``action``, and ``icon``. Because they're so common, Blueprint provides a shorter syntax for menu items with just these properties.

Example
~~~~~~~

.. code-block:: blueprint

   menu {
     item ("label")
     item ("label", "action")
     item ("label", "action", "icon")
   }
