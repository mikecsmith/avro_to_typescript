use crate::errors::GenerationError;
use swc_ecma_ast::ModuleItem;

pub type ModuleItems = Vec<ModuleItem>;

pub type GenerationResult = Result<ModuleItems, GenerationError>;
