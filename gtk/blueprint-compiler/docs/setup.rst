=====
Setup
=====

Setting up Blueprint on a new or existing project
-------------------------------------------------

Using the porting tool
~~~~~~~~~~~~~~~~~~~~~~

Clone `blueprint-compiler <https://gitlab.gnome.org/jwestman/blueprint-compiler>`_
from source. You can install it using ``meson _build`` and ``ninja -C _build install``,
or you can leave it uninstalled.

In your project's directory, run ``blueprint-compiler port`` (or ``<path to blueprint-compiler.py> port``)
to start the porting process. It will walk you through the steps outlined below.
It should work for most projects, but if something goes wrong you may need to
follow the manual steps instead.


Manually
~~~~~~~~

blueprint-compiler works as a meson subproject.

#. Save the following file as ``subprojects/blueprint-compiler.wrap``:

   .. code-block:: cfg

      [wrap-git]
      directory = blueprint-compiler
      url = https://gitlab.gnome.org/jwestman/blueprint-compiler.git
      revision = main
      depth = 1

      [provide]
      program_names = blueprint-compiler

#. Add this to your ``.gitignore``:

   .. code-block::

      /subprojects/blueprint-compiler

#. Rewrite your .ui XML files in blueprint format.

#. Add this to the ``meson.build`` file where you build your GResources:

   .. code-block:: meson.build

      blueprints = custom_target('blueprints',
        input: files(
          # LIST YOUR BLUEPRINT FILES HERE
        ),
        output: '.',
        command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
      )

#. In the same ``meson.build`` file, add this argument to your ``gnome.compile_resources`` command:

   .. code-block:: meson.build

      dependencies: blueprints,

