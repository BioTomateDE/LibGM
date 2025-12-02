use std::{
    io::Write,
    time::{Duration, Instant},
};

use libgm::{
    gamemaker::data::GMData,
    gml::{
        assembly::{assemble_code, disassemble_code},
        instructions::GMInstruction,
    },
    prelude::*,
};

pub fn test_assembler(data: &GMData) -> Result<()> {
    let count = data.codes.len();

    let mut benchmarks: Vec<Benchmark> = Vec::with_capacity(count);

    for i in 0..count {
        let code = &data.codes[i];
        let name = &code.name;

        // Skip child code entries.
        if let Some(b15) = &code.bytecode15_info
            && b15.parent.is_some()
        {
            continue;
        }

        // Print on same line
        print!("\x1B[2K\r({i}/{count}) Disassembling {name}");
        std::io::stdout().flush().unwrap();

        // OR: Print on new line
        //println!("({i}/{count}) Disassembling {name}");

        let start_dis = Instant::now();
        let assembly: String =
            disassemble_code(code, data).with_context(|| format!("disassembling {name:?}"))?;
        let end_dis = Instant::now();

        let start_ass = Instant::now();
        let reconstructed: Vec<GMInstruction> =
            assemble_code(&assembly, data).with_context(|| format!("assembling {name:?}"))?;
        let end_ass = Instant::now();

        benchmarks.push(Benchmark {
            code_name: name,
            disassembler_time: end_dis - start_dis,
            assembler_time: end_ass - start_ass,
        });

        let code = &data.codes[i];
        if code.instructions == reconstructed {
            continue;
        }

        // Assembler (or dissassembler) failed; produced different instructions.
        let orig_len = code.instructions.len();
        let recr_len = reconstructed.len();

        if recr_len != orig_len {
            let diff = recr_len.abs_diff(orig_len);
            let comparison = if recr_len > orig_len { "more" } else { "fewer" };
            println!("Reconstructed code has {diff} {comparison} instructions than the original");
        }

        let lines: Vec<&str> = assembly.split("\n").collect();

        // TODO:prettier diff
        for (index, (original, recreation)) in
            code.instructions.iter().zip(&reconstructed).enumerate()
        {
            if original != recreation {
                let line = lines[index];
                println!("Original: {original:?}");
                println!("Recreation: {recreation:?}");
                println!("Assembly: {line}");
                println!();
            }
        }

        return Err(
            "Assembler produced different instructions than the original (see logs)".into(),
        );
    }

    println!();

    print_statistics(benchmarks);

    Ok(())
}

struct Benchmark<'a> {
    code_name: &'a str,
    disassembler_time: Duration,
    assembler_time: Duration,
}

fn print_statistics(benchmarks: Vec<Benchmark>) {
    let count = benchmarks.len() as u32;
    if count == 0 {
        return;
    }

    let total_time_dis: Duration = benchmarks.iter().map(|b| b.disassembler_time).sum();
    let total_time_ass: Duration = benchmarks.iter().map(|b| b.assembler_time).sum();
    let total_time = total_time_dis + total_time_ass;

    let mean_time_dis = total_time_dis / count;
    let mean_time_ass = total_time_ass / count;
    let mean_time = mean_time_dis + mean_time_ass;

    let slowest_dis = benchmarks
        .iter()
        .max_by(|a, b| a.assembler_time.cmp(&b.disassembler_time))
        .unwrap();
    let slowest_ass = benchmarks
        .iter()
        .max_by(|a, b| a.disassembler_time.cmp(&b.disassembler_time))
        .unwrap();

    println!("===== Statistics =====");
    println!(
        "Total time: {total_time:.3?} (Disassembly: {total_time_dis:.3?}, Assembly: {total_time_ass:.3?})"
    );
    println!(
        "Mean time per code: {mean_time:.3?} (Disassembly: {mean_time_dis:.3?}, Assembly: {mean_time_ass:.3?})"
    );
    println!(
        "Slowest disassembly: {} took {:.3?}",
        slowest_dis.code_name, slowest_dis.disassembler_time,
    );
    println!(
        "Slowest assembly: {} took {:.3?}",
        slowest_ass.code_name, slowest_ass.assembler_time,
    );
    println!();
}
