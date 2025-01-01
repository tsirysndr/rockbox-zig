Overview
========

.. warning::
  .. container:: experimental-admonition

     .. image:: experimental.svg

     **Blueprint is still experimental.** Future versions may have breaking changes, and most GTK tutorials use XML syntax.


Blueprint is a markup language and compiler for GTK 4 user interfaces.

.. toctree::
   :maxdepth: 1
   :caption: Contents:

   setup
   translations
   flatpak
   reference/index
   packaging


.. code-block:: blueprint

   using Gtk 4.0;

   template MyAppWindow : ApplicationWindow {
     default-width: 600;
     default-height: 300;
     title: _("Hello, Blueprint!");

     [titlebar]
     HeaderBar {}

     Label {
       label: bind MyAppWindow.main_text;
     }
   }

Blueprint helps you build user interfaces in GTK quickly and declaratively.
It has modern IDE features like code completion and hover documentation, and
the compiler points out mistakes early on so you can focus on making your app
look amazing.

Features
--------

- **Easy setup.** A porting tool is available to help port your projects from
  XML. The compiler's only dependency is Python, and it can be included as
  a meson subproject. :doc:`See the Setup page for more information. <setup>`
- **Concise syntax.** No more clumsy XML! Blueprint is designed from the ground
  up to match GTK's widget model, including templates, child types, signal
  handlers, and menus.
- **Easy to learn.** The syntax should be very familiar to most people. Take a look at the :doc:`reference <reference/index>` to get started.
- **Modern tooling.** Blueprint ships a `Language Server <https://microsoft.github.io/language-server-protocol/>`_ for IDE integration.

Links
-----

- `Source code <https://gitlab.gnome.org/jwestman/blueprint-compiler>`_
- `Workbench <https://github.com/sonnyp/Workbench>`_ lets you try, preview and export Blueprint
- `GNOME Builder <https://developer.gnome.org/documentation/introduction/builder.html>`_ provides builtin support
- `Vim syntax highlighting plugin by thetek42 <https://github.com/thetek42/vim-blueprint-syntax>`_
- `Vim syntax highlighting plugin by gabmus <https://gitlab.com/gabmus/vim-blueprint>`_
- `GNU Emacs major mode by DrBluefall <https://github.com/DrBluefall/blueprint-mode>`_
- `Visual Studio Code plugin by bodil <https://github.com/bodil/vscode-blueprint>`_

History
-------

1. `Simplify our UI declarative language, a strawman proposal <https://discourse.gnome.org/t/simplify-our-ui-declarative-language-a-strawman-proposal/2913>`_
2. `A Markup Language for GTK <https://www.jwestman.net/2021/10/22/a-markup-language-for-gtk.html>`_
3. `Introducing Blueprint: A New Way to Craft User Interfaces <https://www.jwestman.net/2021/12/02/introducing-blueprint-a-new-way-to-craft-user-interfaces.html>`_
4. `Next Steps for Blueprint <https://www.jwestman.net/2022/04/12/next-steps-for-blueprint.html>`_

Built with Blueprint
--------------------

- `AdwSteamGtk <https://github.com/Foldex/AdwSteamGtk>`_
- `Blurble <https://gitlab.gnome.org/World/Blurble>`_
- `Bottles <https://github.com/bottlesdevs/Bottles>`_
- `Cartridges <https://github.com/kra-mo/cartridges>`_
- `Cassette <https://gitlab.gnome.org/Rirusha/Cassette>`_
- `Cavalier <https://github.com/NickvisionApps/Cavalier>`_
- `Chance <https://zelikos.dev/apps/rollit>`_
- `Commit <https://github.com/sonnyp/Commit/>`_
- `Confy <https://confy.kirgroup.net/>`_
- `Cozy <https://github.com/geigi/cozy>`_
- `Daikhan <https://github.com/flathub/io.gitlab.daikhan.stable>`_
- `Damask <https://gitlab.gnome.org/subpop/damask>`_
- `Denaro <https://github.com/NickvisionApps/Denaro>`_
- `Design <https://github.com/dubstar-04/Design>`_
- `Dev Toolbox <https://github.com/aleiepure/devtoolbox>`_
- `Dialect <https://github.com/dialect-app/dialect>`_
- `Diccionario de la Lengua <https://codeberg.org/rafaelmardojai/diccionario-lengua>`_
- `Doggo <https://gitlab.gnome.org/sungsphinx/Doggo>`_
- `Dosage <https://github.com/diegopvlk/Dosage>`_
- `Dynamic Wallpaper <https://github.com/dusansimic/dynamic-wallpaper>`_
- `Extension Manager <https://github.com/mjakeman/extension-manager>`_
- `Eyedropper <https://github.com/FineFindus/eyedropper>`_
- `favagtk <https://gitlab.gnome.org/johannesjh/favagtk>`_
- `Feeds <https://gitlab.gnome.org/World/gfeeds>`_
- `File Shredder <https://github.com/ADBeveridge/raider>`_
- `Flare <https://gitlab.com/schmiddi-on-mobile/flare>`_
- `Flowtime <https://github.com/Diego-Ivan/Flowtime>`_
- `Fretboard <https://github.com/bragefuglseth/fretboard>`_
- `Frog <https://github.com/TenderOwl/Frog>`_
- `Geopard <https://github.com/ranfdev/Geopard>`_
- `Giara <https://gitlab.gnome.org/World/giara>`_
- `Girens <https://gitlab.gnome.org/tijder/girens>`_
- `Gradience <https://github.com/GradienceTeam/Gradience>`_
- `Graphs <https://gitlab.gnome.org/World/Graphs>`_
- `Health <https://gitlab.gnome.org/World/Health>`_
- `HydraPaper <https://gitlab.com/gabmus/HydraPaper>`_
- `Identity <https://gitlab.gnome.org/YaLTeR/identity>`_
- `Jogger <https://codeberg.org/baarkerlounger/jogger>`_
- `Junction <https://github.com/sonnyp/Junction/>`_
- `Komikku <https://codeberg.org/valos/Komikku>`_
- `Letterpress <https://gitlab.gnome.org/World/Letterpress>`_
- `Login Manager Settings <https://github.com/realmazharhussain/gdm-settings>`_
- `Maniatic Launcher <https://github.com/santiagocezar/maniatic-launcher/>`_
- `Master Key <https://gitlab.com/guillermop/master-key/>`_
- `Misson Center <https://github.com/flathub/io.missioncenter.MissionCenter>`_
- `NewCaw <https://github.com/CodedOre/NewCaw>`_
- `Paper <https://gitlab.com/posidon_software/paper>`_
- `Paper Plane <https://github.com/paper-plane-developers/paper-plane>`_
- `Parabolic <https://github.com/NickvisionApps/Parabolic>`_
- `Passes <https://github.com/pablo-s/passes>`_
- `Pipeline <https://gitlab.com/schmiddi-on-mobile/pipeline>`_
- `Playhouse <https://github.com/sonnyp/Playhouse>`_
- `Plitki <https://github.com/YaLTeR/plitki>`_
- `Raider <https://github.com/ADBeveridge/raider>`_
- `Retro <https://github.com/sonnyp/Retro>`_
- `Solanum <https://gitlab.gnome.org/World/Solanum>`_
- `Sudoku Solver <https://gitlab.com/cyberphantom52/sudoku-solver>`_
- `Swatch <https://gitlab.gnome.org/GabMus/swatch>`_
- `Switcheroo <https://gitlab.com/adhami3310/Switcheroo>`_
- `Tagger <https://github.com/NickvisionApps/Tagger>`_
- `Tangram <https://github.com/sonnyp/Tangram/>`_
- `Text Pieces <https://github.com/liferooter/textpieces>`_
- `Upscaler <https://gitlab.gnome.org/World/Upscaler>`_
- `Video Trimmer <https://gitlab.gnome.org/YaLTeR/video-trimmer>`_
- `Webfont Kit Generator <https://github.com/rafaelmardojai/webfont-kit-generator>`_
- `WhatIP <https://gitlab.gnome.org/GabMus/whatip>`_
- `Who Wants To Be a Millionaire <https://github.com/martinszeltins/who-wants-to-be-a-millionaire/>`_
- `Workbench <https://github.com/sonnyp/Workbench>`_