# If run from a snippet, then binaryninja is already imported.
SNIPPET = "binaryninja" in globals()

from binaryninja import *

import typing
import weggli


class WeggliPlugin(object):
    def __init__(self, bv: BinaryView, print_code=False):
        self.bv = bv
        self.print_code = print_code

    def get_function(self, name: str) -> typing.Optional[Function]:
        for f in self.bv.functions:
            if f.name == name:
                return f
        return None

    def decompile(self, func: Function) -> str:
        Settings().set_string("rendering.hlil.scopingStyle", "bracesNewLine")
        settings = DisassemblySettings()
        settings.set_option(DisassemblyOption.ShowAddress, False)
        settings.set_option(DisassemblyOption.WaitForIL, True)

        obj = lineardisassembly.LinearViewObject.language_representation(
            self.bv, settings
        )
        cursor_end = lineardisassembly.LinearViewCursor(obj)
        cursor_end.seek_to_address(func.highest_address)
        end_lines = self.bv.get_next_linear_disassembly_lines(cursor_end)
        cursor_end.seek_to_address(func.highest_address)
        start_lines = self.bv.get_previous_linear_disassembly_lines(cursor_end)
        lines = start_lines + end_lines

        return "\n".join(
            "".join(
                str(token)
                for token in line.contents.tokens
                if token.type != InstructionTextTokenType.TagToken
            )
            for line in lines
        )

    def xrefs_to(self, f: Function) -> typing.Generator[Function, None, None]:
        for xref in self.bv.get_callers(f.start):
            yield from self.bv.get_functions_containing(xref.address)

    def run_query(self, query: str):
        qt = weggli.parse_query(query)

        identifiers = weggli.identifiers(qt)
        referenced_funcs = list(
            filter(lambda f: f != None, [self.get_function(i) for i in identifiers])
        )

        if len(referenced_funcs) > 0:
            work = set(self.xrefs_to(referenced_funcs[0]))

            for f in referenced_funcs[1:]:
                work.intersection_update(self.xrefs_to(f))

            log_info(f"Searching through {len(work)} functions..")
            for target in work:
                if not target:
                    continue
                code = self.decompile(target)
                if code != None:
                    results = weggli.matches(qt, code)
                    if len(results) > 0:
                        log_info(
                            f"{len(results)} matches in {target.symbol.full_name} @ {hex(target.start)}"
                        )
                        if self.print_code:
                            for r in results:
                                print(weggli.display(r, code))

                else:
                    log_error(f"Decompilation failed for {target.name}. Skipping..")


def run_query(bv: BinaryView):
    w = WeggliPlugin(bv)
    query = get_text_line_input("query", "weggli").decode()
    print(repr(query))
    w.run_query(query)


if not SNIPPET:
    PluginCommand.register("weggli", "Run a weggli query", run_query)
else:
    # bv is injected into globals in the snippet / python console.
    run_query(bv)
