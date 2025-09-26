use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::GMCode;
use crate::gml::decompiler::control_flow::blocks::find_blocks;
use crate::gml::decompiler::control_flow::fragments::find_fragments;
use crate::gml::decompiler::control_flow::ControlFlowGraph;

pub mod control_flow;


pub fn decompile_to_ast(gm_data: &GMData, code_ref: GMRef<GMCode>) -> Result<(), String> {
    let mut cfg = ControlFlowGraph {
        empty_nodes: vec![],
        blocks: vec![],
        fragments: vec![],
        static_inits: vec![],
        short_circuit_blocks: vec![],
        loops: vec![],
    };

    let code = code_ref.resolve(&gm_data.codes.codes)?;
    find_blocks(&mut cfg, &code.instructions);
    find_fragments(&mut cfg, &gm_data, code_ref)?;

    Ok(())
}

