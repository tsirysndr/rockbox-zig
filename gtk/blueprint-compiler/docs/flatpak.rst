=======
Flatpak
=======

Flathub's builders don't allow internet access during the build; everything that
goes into the build must be specified in the manifest. This means meson
submodules won't work. Instead, you need to install blueprint-compiler as
a module in your flatpak manifest:

.. code-block:: json

   {
     "name": "blueprint-compiler",
     "buildsystem": "meson",
     "cleanup": ["*"],
     "sources": [
       {
         "type": "git",
         "url": "https://gitlab.gnome.org/jwestman/blueprint-compiler",
         "tag": "v0.14.0"
       }
     ]
   }

You can keep the submodule configuration--Meson will ignore it if
blueprint-compiler is already installed.