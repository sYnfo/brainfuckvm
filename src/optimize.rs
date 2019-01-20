use parse::Instruction;
use parse::Instruction::*;

use parse::link_jumps;


pub fn optimize(program: &[Instruction]) -> Vec<Instruction> {
    let program = colapse_moves(&program);
    let program = colapse_adds(&program);
    link_jumps(&program)
}

fn colapse_moves(program: &[Instruction]) -> Vec<Instruction> {
    let mut optimized = Vec::with_capacity(program.len());
    let mut move_run = 0;
    for ins in program {
        match ins {
            Move(n) => move_run += n,
            _ => {
                if move_run != 0 {
                    optimized.push(Move(move_run));
                    move_run = 0;
                }
                optimized.push(ins.clone());
            }
        }
    }
    if move_run != 0 {
        optimized.push(Move(move_run));
    }
    optimized
}

fn colapse_adds(program: &[Instruction]) -> Vec<Instruction> {
    let mut optimized = Vec::with_capacity(program.len());
    let mut move_run = 0;
    for ins in program {
        match ins {
            Add(n) => move_run += n,
            _ => {
                if move_run != 0 {
                    optimized.push(Add(move_run));
                    move_run = 0;
                }
                optimized.push(ins.clone());
            }
        }
    }
    if move_run != 0 {
        optimized.push(Add(move_run));
    }
    optimized
}

/*
fn replace_set_zero(program: &[Instruction]) -> Vec<Instruction> {
    let mut optimized = Vec::with_capacity(program.len());
    //let peekable = program.into_iter().multipeek();
    for op in program {
        match op {
            JumpIfZero(_) => optimized.push(op.clone()),
                //if let (Add(-1), JumpIfNotZero(_)) = (program.peek(), program.peek()) {
            _ => optimized.push(op.clone()),
        }
    }
    optimized
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_colapses_moves() {
        let program = vec![Add(1), Move(2), Move(-1), Move(3), Add(1)];
        assert_eq!(colapse_moves(&program), vec![Add(1), Move(4), Add(1)]);
    }

    #[test]
    fn it_colapses_adds() {
        let program = vec![Move(1), Add(2), Add(-1), Add(3), Move(1)];
        assert_eq!(colapse_adds(&program), vec![Move(1), Add(4), Move(1)]);
    }

    #[test]
    fn it_links_jumps() {
        let program = vec![JumpIfZero(4), Add(1), Add(1), JumpIfNotZero(1)];
        assert_eq!(optimize(&program), vec![JumpIfZero(3), Add(2), JumpIfNotZero(1)]);
    }

    /*
    #[test]
    fn it_replaces_set_zero() {
        let program = vec![JumpIfZero(3), Add(-1), JumpIfNotZero(1)];
        assert_eq!(replace_set_zero(&program), vec![SetZero]);
    }
    */
}
