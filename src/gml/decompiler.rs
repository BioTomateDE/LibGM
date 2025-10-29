// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::GMCode;
use crate::gml::decompiler::control_flow::blocks::find_blocks;
use crate::gml::decompiler::control_flow::fragments::find_fragments;
use crate::gml::decompiler::control_flow::loops::find_loops;
use crate::gml::decompiler::control_flow::short_circuits::find_short_circuits;
use crate::gml::decompiler::control_flow::static_inits::find_static_inits;
use crate::gml::decompiler::decompile_context::DecompileContext;
use crate::prelude::*;

mod accessors;
pub mod control_flow;
pub mod decompile_context;
pub mod vm_constants;

pub fn decompile_to_ast(gm_data: &GMData, code_ref: GMRef<GMCode>) -> Result<()> {
    let mut ctx = DecompileContext {
        gm_data,
        nodes: vec![],
        blocks: vec![],
        short_circuit_blocks: vec![],
    };

    let code = code_ref.resolve(&gm_data.codes.codes)?;
    find_blocks(&mut ctx, &code.instructions).context("finding blocks")?;
    // for i in &cfg.blocks {
    //     println!(
    //         "{:>3}..{:<3} ({} | {})  {}",
    //         i.start_address,
    //         i.end_address,
    //         i.instructions
    //             .first()
    //             .map_or("///".to_string(), |x| disassemble_instruction(gm_data, x)
    //                 .unwrap_or("<?>".to_string())),
    //         i.instructions
    //             .last()
    //             .map_or("///".to_string(), |x| disassemble_instruction(gm_data, x)
    //                 .unwrap_or("<?>".to_string())),
    //         i.instructions.len(),
    //     )
    // }
    // // std::process::exit(67);

    // let gamename = std::env::var("FUCKING_GAMENAME").unwrap();
    // let path = format!(
    //     "/home/biotomatede/temp/LibGM/{}/{}",
    //     gamename,
    //     code.name.display(&gm_data.strings)
    // );
    // std::fs::create_dir(format!("/home/biotomatede/temp/LibGM/{gamename}")).ok();
    // let mut s = String::new();
    // for (i, noderef) in ctx.blocks.iter().enumerate() {
    //     let n = noderef.node(&ctx);
    //     s.push_str(&format!("NODE {i} {{\n"));
    //     s.push_str(&format!("  Start: {}\n", n.start_address / 4));
    //     s.push_str(&format!("  End: {}\n", n.end_address / 4));
    //     s.push_str(&format!(
    //         "  Predecessors: {}\n",
    //         n.predecessors
    //             .iter()
    //             .map(|i| i.index.to_string())
    //             .collect::<Vec<_>>()
    //             .join(", ")
    //     ));
    //     let mut succ = vec![];
    //     if let Some(n) = n.successors.branch_target {
    //         succ.push(n);
    //     }
    //     if let Some(n) = n.successors.fall_through {
    //         succ.push(n);
    //     }
    //     s.push_str(&format!(
    //         "  Successors: {}\n",
    //         succ.iter().map(|i| i.index.to_string()).collect::<Vec<_>>().join(", ")
    //     ));
    //     s.push_str("}\n\n");
    // }
    // std::fs::write(path, s).ok();

    // find_fragments(&mut ctx, code_ref).context("finding fragments")?;
    // find_static_inits(&mut ctx).context("finding static inits")?;
    // find_short_circuits(&mut ctx);
    // find_loops(&mut ctx).context("finding loops")?;

    Ok(())
}
