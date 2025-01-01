===========
Expressions
===========

Expressions make your user interface code *reactive*. This means when your
application's data changes, the user interface reacts to the change
automatically.

.. code-block:: blueprint

   label: bind template.account.username;
   /*     ^             ^       ^
          |             creates lookup expressions that are re-evaluated when
          |             the account's username *or* the account itself changes
          |
          binds the `label` property to the expression's output
    */

When a value is bound to an expression using the ``bind`` keyword, the binding
monitors all the object properties that are inputs to the expression, and
reevaluates it if any of them change.

This is a powerful tool for ensuring consistency and simplifying your code.
Rather than pushing changes to the user interface wherever they may occur,
you can define your data model with GObject and let GTK take care of the rest.

.. _Syntax Expression:

Expressions
-----------

.. rst-class:: grammar-block

   Expression = ( :ref:`ClosureExpression<Syntax ClosureExpression>` | :ref:`Literal<Syntax Literal>` | ( '(' Expression ')' ) ) ( :ref:`LookupExpression<Syntax LookupExpression>` | :ref:`CastExpression<Syntax CastExpression>` )*

.. note::

   The grammar above is designed to eliminate `left recursion <https://en.wikipedia.org/wiki/Left_recursion>`_, which can make parsing more complex. In this format, an expression consists of a prefix (such as a literal value or closure invocation) followed by zero or more infix or suffix operators.

Expressions are composed of property lookups and/or closures. Property lookups are the inputs to the expression, and closures provided in application code can perform additional calculations on those inputs.


.. _Syntax LookupExpression:

Lookup Expressions
------------------

.. rst-class:: grammar-block

   LookupExpression = '.' <property::ref:`IDENT<Syntax IDENT>`>

Lookup expressions perform a GObject property lookup on the preceding expression. They are recalculated whenever the property changes, using the `notify signal <https://docs.gtk.org/gobject/signal.Object.notify.html>`_.

The type of a property expression is the type of the property it refers to.


.. _Syntax ClosureExpression:

Closure Expressions
-------------------

.. rst-class:: grammar-block

   ClosureExpression = '$' <name::ref:`IDENT<Syntax IDENT>`> '(' ( :ref:`Expression<Syntax Expression>` ),* ')'

Closure expressions allow you to perform additional calculations that aren't supported in blueprint by writing those calculations as application code. These application-defined functions are created in the same way as :ref:`signal handlers<Syntax Signal>`.

Expressions are only reevaluated when their inputs change. Because blueprint doesn't manage a closure's application code, it can't tell what changes might affect the result. Therefore, closures must be *pure*, or deterministic. They may only calculate the result based on their immediate inputs, not properties of their inputs or outside variables.

Blueprint doesn't know the closure's return type, so closure expressions must be cast to the correct return type using a :ref:`cast expression<Syntax CastExpression>`.


.. _Syntax CastExpression:

Cast Expressions
----------------

.. rst-class:: grammar-block

   CastExpression = 'as' '<' :ref:`TypeName<Syntax TypeName>` '>'

Cast expressions allow Blueprint to know the type of an expression when it can't otherwise determine it. This is necessary for closures and for properties of application-defined types.

.. code-block:: blueprint

   // Cast the result of the closure so blueprint knows it's a string
   label: bind $my_closure() as <string>