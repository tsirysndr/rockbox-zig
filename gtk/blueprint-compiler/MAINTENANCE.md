## Releasing a new version

1. Look at the git log since the previous release. Note every significant change
in the NEWS file.
2. Update the version number, according to semver:
  - At the top of meson.build
  - In docs/flatpak.rst
3. Make a new commit with just these two changes. Use `Release v{version}` as the commit message. Tag the commit as `v{version}` and push the tag.
4. Create a "Post-release version bump" commit.
5. Go to the Releases page in GitLab and create a new release from the tag.
6. Announce the release through relevant channels (Twitter, TWIG, etc.)

## Related projects

Blueprint is supported by the following syntax highlighters. If changes are made to the syntax, remember to update these projects as well.

- Pygments (https://github.com/pygments/pygments/blob/master/pygments/lexers/blueprint.py)
- GtkSourceView (https://gitlab.gnome.org/GNOME/gtksourceview/-/blob/master/data/language-specs/blueprint.lang)