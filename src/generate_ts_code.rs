use std::rc::Rc;

use swc_common::SourceMap;
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};

use crate::{errors::GenerationError, types::GenerationResult};

pub fn generate_ts_code(module_items: GenerationResult) -> Result<String, GenerationError> {
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

        for item in module_items.map_err(GenerationError::from)? {
            emitter.emit_module_item(&item)?;
        }
    }

    String::from_utf8(buf).map_err(GenerationError::from)
}
