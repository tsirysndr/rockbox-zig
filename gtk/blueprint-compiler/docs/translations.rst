============
Translations
============

Blueprint files can be translated with xgettext. To mark a string as translated,
use the following syntax:

.. code-block:: blueprint

   _("translated string")

You'll need to use a few xgettext flags so it will recognize the format:

.. code-block::

   --from-code=UTF-8
   --add-comments
   --keyword=_
   --keyword=C_:1c,2

If you're using Meson's `i18n module <https://mesonbuild.com/i18n-module.html#i18ngettext>`_, you can use the 'glib' preset:

.. code-block:: meson.build

   i18n.gettext('package name', preset: 'glib')

Contexts
--------

Unlike most other translation libraries, which use a separate string key,
gettext uses the English translation as the key. This is great for effortlessly
adding new strings--you just mark them as needing translation--but it can cause
conflicts. Two strings that are the same in English, but appear in different
contexts, might be different in another language! To disambiguate, use ``C_``
instead of ``_`` and add a context string as the first argument:

.. code-block:: blueprint

   C_("shortcuts window", "Quit")

The context string will be shown to translators, but will not appear in the UI.