use parse::Instruction;
use parse::Instruction::*;

pub fn annotate(program: &[Instruction], profile: Option<Vec<u32>>) -> String {
    let mut result = String::new();
    let line_no_width: usize = (program.len() as f64).log(10.0) as usize + 1;
    let mut indent = 0;
    for (line_no, op) in program.iter().enumerate() {
        let decoration = match op {
            JumpIfZero(_) => "⬐",
            JumpIfNotZero(_) => "⬑",
            _ => " ",
        };
        if let JumpIfZero(_) = op {
            indent += 3;
        };
        let mut line = format!("{0:<1$}  {2:<3$}{4}{5:?}",
            line_no + 1, line_no_width,
            "", indent,
            decoration,
            op,
        );
        if profile.is_some() {
            let count = profile.as_ref().unwrap()[line_no];
            line.push_str(&format!(" {}", count));
        };
        line.push('\n');
        result.push_str(&line);
        if let JumpIfNotZero(_) = op {
            indent -= 3;
        };
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_annotates() {
        let program = vec![Add(1)];
        assert_eq!(annotate(&program, None), "1   Add(1)\n");
    }

    #[test]
    fn it_annotates_with_profile() {
        let program = vec![Add(1)];
        assert_eq!(annotate(&program, Some(vec![10])), "1   Add(1) 10\n");
    }
}
