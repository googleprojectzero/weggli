use binaryninja::binaryview::{BinaryView, BinaryViewExt};
use binaryninja::disassembly::{DisassemblyOption, DisassemblySettings};
use binaryninja::function::Function;
use binaryninja::linearview::{LinearViewCursor, LinearViewObject};

use std::fmt::Write;
use std::path::Path;

use binaryninja::rc::*;

pub struct Decompiler {
    view: Ref<BinaryView>,
}

impl Decompiler {
    pub fn from_file(path: &Path) -> Self {
        let view = binaryninja::open_view(path).expect("Couldn't open file");
        Self { view }
    }

    pub fn functions(&self) -> Array<Function> {
        self.view.functions()
    }

    pub fn decompile_function(&self, function: &Function) -> String {
        let settings = DisassemblySettings::new();
        settings.set_option(DisassemblyOption::ShowAddress, false);
        settings.set_option(DisassemblyOption::WaitForIL, true);

        let linearview = LinearViewObject::language_representation(&self.view, &settings);

        let mut cursor = LinearViewCursor::new(&linearview);
        cursor.seek_to_address(function.highest_address());

        let first_lines = self
            .view
            .get_previous_linear_disassembly_lines(&mut cursor.duplicate());
        let next_lines = self.view.get_next_linear_disassembly_lines(&mut cursor);

        let lines = first_lines.iter().chain(next_lines.iter());

        let mut decompilation = String::new();

        for line in lines {
            writeln!(decompilation, "{}", line.as_ref()).unwrap();
        }

        decompilation
    }
}
