#!/usr/bin/env python3

import json
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path

__all__ = ["get_docs_section"]

DOCS_ROOT = "https://jwestman.pages.gitlab.gnome.org/blueprint-compiler"


sections: dict[str, "Section"] = {}


@dataclass
class Section:
    link: str
    lines: str

    def to_json(self):
        return {
            "content": rst_to_md(self.lines),
            "link": self.link,
        }


def load_reference_docs():
    for filename in Path(os.path.dirname(__file__), "reference").glob("*.rst"):
        with open(filename) as f:
            section_name = None
            lines = []

            def close_section():
                if section_name:
                    html_file = re.sub(r"\.rst$", ".html", filename.name)
                    anchor = re.sub(r"[^a-z0-9]+", "-", section_name.lower())
                    link = f"{DOCS_ROOT}/reference/{html_file}#{anchor}"
                    sections[section_name] = Section(link, lines)

            for line in f:
                if m := re.match(r"\.\.\s+_(.*):", line):
                    close_section()
                    section_name = m.group(1)
                    lines = []
                else:
                    lines.append(line)

            close_section()


# This isn't a comprehensive rST to markdown converter, it just needs to handle the
# small subset of rST used in the reference docs.
def rst_to_md(lines: list[str]) -> str:
    result = ""

    def rst_to_md_inline(line):
        line = re.sub(r"``(.*?)``", r"`\1`", line)
        line = re.sub(
            r":ref:`(.*?)<(.*?)>`",
            lambda m: f"[{m.group(1)}]({sections[m.group(2)].link})",
            line,
        )
        line = re.sub(r"`([^`]*?) <([^`>]*?)>`_", r"[\1](\2)", line)
        return line

    i = 0
    n = len(lines)
    heading_levels = {}

    def print_block(lang: str = "", code: bool = True, strip_links: bool = False):
        nonlocal result, i
        block = ""
        while i < n:
            line = lines[i].rstrip()
            if line.startswith("   "):
                line = line[3:]
            elif line != "":
                break

            if strip_links:
                line = re.sub(r":ref:`(.*?)<(.*?)>`", r"\1", line)

            if not code:
                line = rst_to_md_inline(line)

            block += line + "\n"
            i += 1

        if code:
            result += f"```{lang}\n{block.strip()}\n```\n\n"
        else:
            result += block

    while i < n:
        line = lines[i].rstrip()
        i += 1
        if line == ".. rst-class:: grammar-block":
            print_block("text", strip_links=True)
        elif line == ".. code-block:: blueprint":
            print_block("blueprint")
        elif line == ".. note::":
            result += "#### Note\n"
            print_block(code=False)
        elif m := re.match(r"\.\. image:: (.*)", line):
            result += f"![{m.group(1)}]({DOCS_ROOT}/_images/{m.group(1)})\n"
        elif i < n and re.match(r"^((-+)|(~+)|(\++))$", lines[i]):
            level_char = lines[i][0]
            if level_char not in heading_levels:
                heading_levels[level_char] = max(heading_levels.values(), default=1) + 1
            result += (
                "#" * heading_levels[level_char] + " " + rst_to_md_inline(line) + "\n"
            )
            i += 1
        else:
            result += rst_to_md_inline(line) + "\n"

    return result


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: collect_sections.py <output_file>")
        sys.exit(1)

    outfile = sys.argv[1]

    load_reference_docs()

    # print the sections to a json file
    with open(outfile, "w") as f:
        json.dump(
            {name: section.to_json() for name, section in sections.items()}, f, indent=2
        )
