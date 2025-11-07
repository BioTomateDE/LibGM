// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::gamemaker::elements::code::{GMCodeValue, GMInstruction, GMPushInstruction};
use crate::gml::decompiler::decompile_context::DecompileContext;

pub fn find_short_circuits(ctx: &mut DecompileContext) {
    let pre_bytecode15: bool = ctx.gm_data.general_info.bytecode_version < 15;

    // Identify and restructure short circuits
    for block_ref in &ctx.blocks {
        let is_short_circuit: bool = match block_ref.block(ctx).instructions {
            [GMInstruction::PushImmediate(_)] if pre_bytecode15 => true,
            [GMInstruction::Push(GMPushInstruction { value: GMCodeValue::Int16(_) })] => true,
            _ => false,
        };
        if is_short_circuit {
            ctx.short_circuit_blocks.push(*block_ref);
        }
    }
}
