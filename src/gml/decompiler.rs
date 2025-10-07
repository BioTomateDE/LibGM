use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::GMCode;
use crate::gml::decompiler::control_flow::ControlFlowGraph;
use crate::gml::decompiler::control_flow::blocks::find_blocks;
use crate::gml::decompiler::control_flow::fragments::find_fragments;
use crate::gml::decompiler::control_flow::loops::find_loops;
use crate::gml::decompiler::control_flow::short_circuits::find_short_circuits;
use crate::gml::decompiler::control_flow::static_inits::find_static_inits;
use crate::gml::decompiler::decompile_context::DecompileContext;
use crate::gml::disassembler::disassemble_instruction;
use crate::prelude::*;

pub mod control_flow;
pub mod decompile_context;
pub mod vm_constants;

pub fn decompile_to_ast(gm_data: &GMData, code_ref: GMRef<GMCode>) -> Result<()> {
    let mut cfg = ControlFlowGraph {
        context: DecompileContext { gm_data },
        empty_nodes: vec![],
        blocks: vec![],
        fragments: vec![],
        static_inits: vec![],
        short_circuit_blocks: vec![],
        loops: vec![],
    };

    let code = code_ref.resolve(&gm_data.codes.codes)?;
    find_blocks(&mut cfg, &code.instructions)?;
    for i in &cfg.blocks {
        println!(
            "{:>3}..{:<3} ({} | {})  {}",
            i.start_address,
            i.end_address,
            i.instructions
                .first()
                .map_or("///".to_string(), |x| disassemble_instruction(gm_data, x)
                    .unwrap_or("<?>".to_string())),
            i.instructions
                .last()
                .map_or("///".to_string(), |x| disassemble_instruction(gm_data, x)
                    .unwrap_or("<?>".to_string())),
            i.instructions.len(),
        )
    }
    // std::process::exit(67);

    find_fragments(&mut cfg, code_ref)?;
    find_static_inits(&mut cfg)?;
    find_short_circuits(&mut cfg);
    find_loops(&mut cfg)?;

    Ok(())
}
