use libgm::gamemaker::data::GMData;
use libgm::gml::decompiler::decompile_to_ast;
use libgm::prelude::*;

pub fn test_decompiler(data: &GMData) -> Result<()> {
    for (i, code) in data.codes.iter().enumerate() {
        if let Some(b15) = &code.bytecode15_info
            && b15.parent.is_some()
        {
            continue;
        }

        let name = &code.name;
        println!("({}/{}) Decompiling: {:<64}", i + 1, data.codes.len(), name);

        if code.instructions.is_empty() {
            continue;
        }

        decompile_to_ast(data, i.into()).with_context(|| format!("decompiling {name}"))?;
    }
    Ok(())
}
