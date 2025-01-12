# Blueprint

A markup language for GTK user interface files.

## Motivation

GtkBuilder XML format is quite verbose, and many app developers don't like
using WYSIWYG editors for creating UIs. Blueprint files are intended to be a
concise, easy-to-read format that makes it easier to create and edit GTK UIs.

Internally, it compiles to GtkBuilder XML as part of an app's build system. It
adds no new features, just makes the features that exist more accessible.

Another goal is to have excellent developer tooling--including a language
server--so that less knowledge of the format is required. Hopefully this will
increase adoption of cool advanced features like GtkExpression.

## Example

Here is what [the libshumate demo's UI definition](https://gitlab.gnome.org/GNOME/libshumate/-/blob/main/demos/shumate-demo-window.ui)
looks like ported to this new format:

```
using Gtk 4.0;
using Shumate 1.0;

template ShumateDemoWindow : Gtk.ApplicationWindow {
  can-focus: yes;
  title: _("Shumate Demo");
  default-width: 800;
  default-height: 600;

  [titlebar]
  Gtk.HeaderBar {
    Gtk.DropDown layers_dropdown {
      notify::selected => on_layers_dropdown_notify_selected() swapped;
    }
  }

  Gtk.Overlay overlay {
    vexpand: true;
    Shumate.Map map {}

    [overlay]
    Shumate.Scale scale {
      halign: start;
      valign: end;
    }

    [overlay]
    Gtk.Box {
      orientation: vertical;
      halign: end;
      valign: end;

      Shumate.Compass compass {
        halign: end;
        map: map;
      }
      Shumate.License license {
        halign: end;
      }
    }
  }
}
```

## Editors

[Workbench](https://github.com/sonnyp/Workbench) and [GNOME Builder](https://apps.gnome.org/app/org.gnome.Builder/) have builtin support for Blueprint.

Vim

- [Syntax highlighting by thetek42](https://github.com/thetek42/vim-blueprint-syntax)
- [Syntax highlighting by gabmus](https://gitlab.com/gabmus/vim-blueprint)

GNU Emacs

- [Major mode by DrBluefall](https://github.com/DrBluefall/blueprint-mode)

Visual Studio Code

- [Blueprint Language Plugin by bodil](https://github.com/bodil/vscode-blueprint)

## Donate

You can support my work on GitHub Sponsors! <https://github.com/sponsors/jameswestman>

## Getting in Touch

Matrix room: [#blueprint-language:matrix.org](https://matrix.to/#/#blueprint-language:matrix.org)

## License

Copyright (C) 2021 James Westman <james@jwestman.net>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
