use std::rc::Rc;

use swc_common::SourceMap;
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};

use crate::{errors::GenerationError, types::ModuleItems};

pub fn generate_ts_code(module_items: ModuleItems) -> Result<String, GenerationError> {
    let cm = Rc::new(SourceMap::default());

    let mut buf = vec![];

    {
        let writer = JsWriter::new(Rc::clone(&cm), "\n", &mut buf, None);

        let mut emitter = Emitter {
            cfg: Default::default(),
            cm,
            wr: writer,
            comments: None,
        };

        for item in &module_items {
            emitter.emit_module_item(item)?;
        }
    }

    String::from_utf8(buf).map_err(GenerationError::from)
}
