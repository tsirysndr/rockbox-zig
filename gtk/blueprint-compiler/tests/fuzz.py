import os
import sys

from pythonfuzz.main import PythonFuzz

from blueprintcompiler.outputs.xml import XmlOutput

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

from blueprintcompiler import decompiler, gir, parser, tokenizer, utils
from blueprintcompiler.completions import complete
from blueprintcompiler.errors import (
    CompileError,
    CompilerBugError,
    MultipleErrors,
    PrintableError,
)
from blueprintcompiler.tokenizer import Token, TokenType, tokenize


@PythonFuzz
def fuzz(buf):
    try:
        blueprint = buf.decode("ascii")

        tokens = tokenizer.tokenize(blueprint)
        ast, errors, warnings = parser.parse(tokens)

        xml = XmlOutput()
        if errors is None and ast is not None:
            xml.emit(ast)
    except CompilerBugError as e:
        raise e
    except PrintableError:
        pass
    except UnicodeDecodeError:
        pass


if __name__ == "__main__":
    # Make sure Gtk 4.0 is accessible, otherwise every test will fail on that
    # and nothing interesting will be tested
    gir.get_namespace("Gtk", "4.0")

    fuzz()
