# v0.14.0

## Added
- Added a warning for unused imports.
- Added an option to not print the diff when formatting with the CLI. (Gregor Niehl)
- Added support for building Gtk.ColumnViewRow, Gtk.ColumnViewCell, and Gtk.ListHeader widgets with Gtk.BuilderListItemFactory.
- Added support for the `after` keyword for signals. This was previously documented but not implemented. (Gregor Niehl)
- Added support for string arrays. (Diego Augusto)
- Added hover documentation for properties in lookup expressions.
- The decompiler supports action widgets, translation domains, `typeof<>` syntax, and expressions. It also supports extension syntax for Adw.Breakpoint, Gtk.BuilderListItemFactory, Gtk.ComboBoxText, Gtk.SizeGroup, and Gtk.StringList.
- Added a `decompile` subcommand to the CLI, which decompiles an XML .ui file to blueprint.
- Accessibility relations that allow multiple values are supported using list syntax. (Julian Schmidhuber)

## Changed
- The decompiler sorts imports alphabetically.
- Translatable strings use `translatable="yes"` instead of `translatable="true"` for compatibility with xgettext. (Marco Köpcke)
- The first line of the documentation is shown in the completion list when using the language server. (Sonny Piers)
- Object autocomplete uses a snippet to add the braces and position the cursor inside them. (Sonny Piers)
- The carets in the CLI diagnostic output now span the whole error message up to the end of the first line, rather than just the first character.
- The decompiler emits double quotes, which are compatible with gettext.

## Fixed
- Fixed deprecation warnings in the language server.
- The decompiler no longer duplicates translator comments on properties.
- Subtemplates no longer output a redundant `@generated` comment.
- When extension syntax from a library that is not available is used, the compiler emits an error instead of crashing.
- The language server reports semantic token positions correctly. (Szepesi Tibor)
- The decompiler no longer emits the deprecated `bind-property` syntax. (Sonny Piers)
- Fixed the tests when used as a Meson subproject. (Benoit Pierre)
- Signal autocomplete generates correct syntax. (Sonny Piers)
- The decompiler supports templates that do not specify a parent class. (Sonny Piers)
- Adw.Breakpoint setters that set a property on the template no longer cause a crash.
- Fixed type checking with templates that do not have a parent class.
- Fixed online documentation links for interfaces.
- The wording of edit suggestions is fixed for insertions and deletions.
- When an input file uses tabs instead of spaces, the diagnostic output on the CLI aligns the caret correctly.
- The decompiler emits correct syntax when a property binding refers to the template object.

## Documentation
- Fixed typos in "Built with Blueprint" section. (Valéry Febvre, Dexter Reed)

# v0.12.0

## Added

- Add support for Adw.AlertDialog (Sonny Piers)
- Emit warnings for deprecated APIs - lsp and compiler
- lsp: Document symbols
- lsp: "Go to definition" (ctrl+click)
- lsp: Code action for "namespace not imported" diagnostics, that adds the missing import
- Add a formatter - cli and lsp (Gregor Niehl)
- Support for translation domain - see documentation
- cli: Print code actions in error messages

## Changed

- compiler: Add a header notice mentionning the file is generated (Urtsi Santsi)
- decompiler: Use single quotes for output

## Fixed

- Fixed multine strings support with the escape newline character
- lsp: Fixed the signal completion, which was missing the "$"
- lsp: Fixed property value completion  (Ivan Kalinin)
- lsp: Added a missing semantic highlight (for the enum in Gtk.Scale marks)
- Handle big endian bitfields correctly (Jerry James)
- batch-compile: Fix mixing relative and absolute paths (Marco Köpcke )

## Documentation

- Fix grammar for bindings
- Add section on referencing templates

# v0.10.0

## Added

- The hover documentation now includes a link to the online documentation for the symbol, if available.
- Added hover documentation for the Adw.Breakpoint extensions, `condition` and `setters`.

## Changed

- Decompiling an empty file now produces an empty file rather than an error. (AkshayWarrier)
- More relevant documentation is shown when hovering over an identifier literal (such as an enum value or an object ID).

## Fixed

- Fixed an issue with the language server not conforming the spec. (seshotake)
- Fixed the signature section of the hover documentation for properties and signals.
- Fixed a bug where documentation was sometimes shown for a different symbol with the same name.
- Fixed a bug where documentation was not shown for accessibility properties that contain `-`.
- Number literals are now correctly parsed as floats if they contain a `.`, even if they are divisible by 1.

## Removed

- The `bind-property` keyword has been removed. Use `bind` instead. The old syntax is still accepted with a warning.

## Documentation

- Fixed the grammar for Extension, which was missing ExtAdwBreakpoint.


# v0.8.1

## Breaking Changes

- Duplicates in a number of places are now considered errors. For example, duplicate flags in several places, duplicate
  strings in Gtk.FileFilters, etc.

## Fixed

- Fixed a number of bugs in the XML output when using `template` to refer to the template object.

## Documentation

- Fixed the example for ExtListItemFactory

# v0.8.0

## Breaking Changes

- A trailing `|` is no longer allowed in flags.
- The primitive type names `gboolean`, `gchararray`, `gint`, `gint64`, `guint`, `guint64`, `gfloat`, `gdouble`, `utf8`, and `gtype` are no longer permitted. Use the non-`g`-prefixed versions instead.
- Translated strings may no longer have trailing commas.

## Added

- Added cast expressions, which are sometimes needed to specify type information in expressions.
- Added support for closure expressions.
- Added the `--typelib-path` command line argument, which allows adding directories to the search path for typelib files.
- Added custom compile and decompile commands to the language server. (Sonny Piers)
- Added support for [Adw.MessageDialog](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1-latest/class.MessageDialog.html#adwmessagedialog-as-gtkbuildable) custom syntax.
- Added support for inline sub-templates for [Gtk.BuilderListItemFactory](https://docs.gtk.org/gtk4/class.BuilderListItemFactory.html). (Cameron Dehning)
- Added support for [Adw.Breakpoint](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Breakpoint.html) custom syntax.
- Added a warning when an object ID might be confusing.
- Added support for [Gtk.Scale](https://docs.gtk.org/gtk4/class.Scale.html#gtkscale-as-gtkbuildable) custom syntax.

## Changed

Some of these changes affect syntax, but the old syntax is still accepted with a purple "upgrade" warning, so they are not breaking changes yet. In editors that support code actions, such as Visual Studio Code, the blueprint language server can automatically fix these warnings.

- The XML output uses the integer value rather than GIR name for enum values.
- Compiler errors are now printed to stderr rather than stdout. (Sonny Piers)
- Introduced `$` to indicate types or callbacks that are provided in application code.
  - Types that are provided by application code are now begin with a `$` rather than a leading `.`.
  - The handler name in a signal is now prefixed with `$`.
  - Closure expressions, which were added in this version, are also prefixed with `$`.
- When a namespace is not found, errors are supressed when the namespace is used.
- The compiler bug message now reports the version of blueprint-compiler.
- The `typeof` syntax now uses `<>` instead of `()` to match cast expressions.
- Menu sections and subsections can now have an ID.
- The interactive porting tool now ignores hidden folders. (Sonny Piers)
- Templates now use the typename syntax rather than an ID to specify the template's class. In most cases, this just means adding a `$` prefix to the ID, but for GtkListItem templates it should be shortened to ListItem (since the Gtk namespace is implied). The template object is now referenced with the `template` keyword rather than with the ID.

## Fixed

- Fixed a bug in the language server's acceptance of text change commands. (Sonny Piers)
- Fixed a bug in the display of diagnostics when the diagnostic is at the beginning of a line.
- Fixed a crash that occurred when dealing with array types.
- Fixed a bug that prevented Gio.File properties from being settable.

## Documentation

- Added a reference section to the documentation. This replaces the Examples page with a detailed description of each syntax feature, including a formal specification of the grammar.

# v0.6.0

## Breaking Changes
- Quoted and numeric literals are no longer interchangeable (e.g. `"800"` is no longer an accepted value for an
  integer type).
- Boxed types are now type checked.

## Added
- There is now syntax for `GType` literals: the `typeof()` pseudo-function. For example, list stores have an `item-type`
  property which is now specifiable like this: `item-type: typeof(.MyDataModel)`. See the documentation for more details.

## Changed
- The language server now logs to stderr.

## Fixed
- Fix the build on Windows, where backslashes in paths were not escaped. (William Roy)
- Remove the syntax for specifying menu objects inline, since it does not work.
- Fix a crash in the language server that was triggered in files with incomplete `using Gtk 4.0;` statements.
- Fixed compilation on big-endian systems.
- Fix an issue in the interactive port tool that would lead to missed files. (Frank Dana)

## Documentation
- Fix an issue for documentation contributors where changing the documentation files would not trigger a rebuild.
- Document the missing support for Gtk.Label `<attributes>`, which is intentional, and recommend alternatives. (Sonny
  Piers)
- Add a prominent warning that Blueprint is still experimental


# v0.4.0

## Added
- Lookup expressions
- With the language server, hovering over a diagnostic message now shows any
  associated hints.

## Changed
- The compiler now uses .typelib files rather than XML .gir files, which reduces
  dependencies and should reduce compile times by about half a second.

## Fixed
- Fix the decompiler/porting tool not importing the Adw namespace when needed
- Fix a crash when trying to compile an empty file
- Fix parsing of number tokens
- Fix a bug where action widgets did not work in templates
- Fix a crash in the language server that occurred when a `using` statement had
no version
- If a compiler bug is reported, the process now exits with a non-zero code
