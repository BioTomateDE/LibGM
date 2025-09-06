use std::collections::HashSet;
use crate::gml::decompiler::ast::{Expression, Statement};
use crate::gml::decompiler::blocks::{BasicBlock, Successors};


#[derive(Debug)]
pub enum BlockType {
    Normal,
    UnconditionalBranch,
    ConditionalHeader,
    LoopHeader,
    Entry,
    Exit,
}

impl BasicBlock<'_> {
    pub fn classify(&self) -> BlockType {
        if self.predecessors.is_empty() {
            return BlockType::Entry
        }

        match self.successors {
            Successors::None => BlockType::Exit,
            Successors::Next => BlockType::Normal,
            Successors::UnconditionalBranch(_) => BlockType::UnconditionalBranch,
            Successors::ConditionalBranch(_) => BlockType::ConditionalHeader,
        }
    }
}


pub fn idk(blocks: &[BasicBlock], start: usize) -> HashSet<usize> {
    let mut visited = HashSet::with_capacity(blocks.len());
    let mut alt_path = HashSet::new();
    let mut i: usize = start;

    while let Some(block) = blocks.get(i) {
        visited.insert(i);

        match block.successors {
            Successors::None => break,
            Successors::Next => i += 1,
            Successors::UnconditionalBranch(target) => i = target,
            Successors::ConditionalBranch(target) => {
                if !alt_path.is_empty() {
                    alt_path = idk(blocks, target);
                }
                i += 1;
            }
        }

        let common: Vec<&usize> = alt_path.intersection(&visited).collect();
        if common.is_empty() {
            continue
        }
        assert_eq!(common.len(), 1);
        let common = *common[0];
        panic!("found merge point: {common}")

    }
    visited
}


// fn structure_blocks(blocks: &[BasicBlock]) -> (Vec<Statement>, Option<usize>) {
//     let mut statements = Vec::new();
//     let mut current_index = 0;
//
//     while current_index < blocks.len() {
//         let block = &blocks[current_index];
//         if let Some((condition, then_start, else_start)) = detect_if(block, blocks) {
//             // Structure the then and else branches
//             let (then_statements, then_next) = structure_blocks(&blocks[then_start..]);
//             let (else_statements, else_next) = structure_blocks(&blocks[else_start..]);
//
//             // Determine the merge point after the if statement
//             let merge_point = match (then_next, else_next) {
//                 (Some(tn), Some(en)) if tn == en => Some(tn),
//                 (Some(tn), None) => Some(tn),
//                 (None, Some(en)) => Some(en),
//                 (None, None) => None,
//                 _ => None, // Handle unstructured code appropriately
//             };
//
//             statements.push(Statement::If {
//                 condition,
//                 then_block: then_statements,
//                 else_block: else_statements,
//             });
//
//             // Move to the merge point or stop if there is none
//             current_index = merge_point.unwrap_or(blocks.len());
//         // } else if let Some(header_index) = block.loop_header {
//         //     // Check if the current block is the loop header
//         //     if current_index == header_index {
//         //         // Parse loop init, condition, and increment from instructions
//         //         let (init, condition, increment) = parse_loop_components(block, blocks);
//         //         let body = structure_blocks(&blocks[current_index+1..]); // Body starts after header
//         //         statements.push(Statement::Loop {
//         //             init,
//         //             condition,
//         //             increment,
//         //             body: body.0,
//         //         });
//         //         // Skip the loop body; need to find the end of the loop
//         //         current_index = skip_loop_body(blocks, current_index);
//         //     } else {
//         //         // Not the header, so treat as a regular block
//         //         statements.push(Statement::Block(Vec::new()));
//         //         current_index += 1;
//         //     }
//         } else {
//             // Handle other block types (e.g., basic blocks, loops)
//             // For now, push a placeholder statement and move to the next block
//             statements.push(Statement::Block(Vec::new()));
//             current_index += 1;
//         }
//     }
//
//     (statements, None) // Adjust based on actual control flow
// }
//
//
// fn detect_if(block: &BasicBlock, blocks: &[BasicBlock]) -> Option<(Expression, usize, usize)> {
//     if let Successors::ConditionalBranch(else_target) = &block.successors {
//         let condition = parse_condition(block.instructions); // Parse condition from instructions
//         let then_start = block.block_index + 1; // Then branch starts at the next block
//         Some((condition, then_start, *else_target))
//     } else {
//         None
//     }
// }
//
//
// // fn detect_loops(blocks: &mut [BasicBlock]) {
// //     // Implement a DFS-based loop detection algorithm
// //     // For each block, check if it has a successor that is a predecessor (indicating a cycle)
// //     // Mark the loop header block with `loop_header: Some(header_index)`
// //     // This is a simplified version; actual implementation may require dominance analysis
// //     for i in 0..blocks.len() {
// //         for successor in blocks[i].successors.get_targets() {
// //             if successor <= i {
// //                 blocks[successor].loop_header = Some(successor);
// //             }
// //         }
// //     }
// // }
//
