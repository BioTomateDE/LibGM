use crate::gamemaker::elements::code::{GMCodeValue, GMInstruction, GMPushInstruction};
use crate::gml::decompiler::control_flow::{ControlFlowGraph, NodeRef};

pub fn find_short_circuits(cfg: &mut ControlFlowGraph) {
    let pre_bytecode15: bool = cfg.context.gm_data.general_info.bytecode_version < 15;

    // Identify and restructure short circuits
    for (i, block) in cfg.blocks.iter().enumerate() {
        let is_short_circuit: bool = match block.instructions {
            [GMInstruction::PushImmediate(_)] if pre_bytecode15 => true,
            [GMInstruction::Push(GMPushInstruction { value: GMCodeValue::Int16(_) })] => true,
            _ => false,
        };
        if is_short_circuit {
            cfg.short_circuit_blocks.push(NodeRef::block(i));
        }
    }
}
