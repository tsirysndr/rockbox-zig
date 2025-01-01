====================
For Distro Packagers
====================

blueprint-compiler is a build tool that converts UI definitions written in
Blueprint into XML files that are installed with the app and that GTK can read.
So for most applications that use blueprint-compiler, it is a build dependency.
It is a Python program, but like most GNOME-related projects, it uses
`Meson <https://mesonbuild.com>`_ as its build system.

GObject Introspection
~~~~~~~~~~~~~~~~~~~~~

Blueprint files can import GObject Introspection namespaces like this:

.. code-block:: blueprint

   using Gtk 4.0;
   using Adw 1;

To compile a blueprint file, ``.typelib`` files for all of the imported
namespaces must be installed. All blueprint files must import Gtk 4.0, so
``Gtk-4.0.typelib`` is effectively a runtime dependency of blueprint-compiler.
blueprint-compiler also depends on pygobject, because it uses GIRepository
to determine the search path for typelib files.

So, if a package uses blueprint-compiler, its build dependencies should include
the typelib files for any namespaces imported in its blueprint files. (Note
that many apps also have the same typelib files as runtime dependencies,
separately from blueprint).

In addition, the blueprint language server uses ``.gir`` files to provide
documentation on hover. Some distros package these files separately from the
main package (e.g. in a ``-devel`` package). The language server will not crash
if these files are not present, but for a good user experience you should make
sure they are installed.