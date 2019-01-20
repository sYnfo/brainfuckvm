extern crate test;

#[derive(PartialEq, Debug, Clone)]
pub enum Instruction {
    Move(isize),
    Add(isize),
    Print,
    Read,
    JumpIfZero(usize),
    JumpIfNotZero(usize),
    SetZero,
}

pub fn parse(program: &[char]) -> Vec<Instruction> {
    link_jumps(
        &program
            .iter()
            .filter_map(|tok| match tok {
                '>' => Some(Instruction::Move(1)),
                '<' => Some(Instruction::Move(-1)),
                '+' => Some(Instruction::Add(1)),
                '-' => Some(Instruction::Add(-1)),
                '.' => Some(Instruction::Print),
                ',' => Some(Instruction::Read),
                '[' => Some(Instruction::JumpIfZero(0)),
                ']' => Some(Instruction::JumpIfNotZero(0)),
                _ => None,
            }).collect::<Vec<_>>(),
    )
}

pub fn link_jumps(program: &[Instruction]) -> Vec<Instruction> {
    let mut jump_stack = Vec::new();
    let mut linked_program: Vec<Instruction> = Vec::new();
    for (i, instruction) in program.iter().enumerate() {
        match instruction {
            Instruction::JumpIfZero(_) => {
                jump_stack.push(i);
                linked_program.push(instruction.clone());
            }
            Instruction::JumpIfNotZero(_) => {
                let prev_jump = jump_stack.pop().expect("Unbalanced program");
                linked_program[prev_jump] = Instruction::JumpIfZero(i + 1);
                linked_program.push(Instruction::JumpIfNotZero(prev_jump + 1));
            }
            _ => linked_program.push(instruction.clone()),
        }
    }
    if !linked_program
        .iter()
        .filter(|tok| **tok == Instruction::JumpIfZero(0))
        .collect::<Vec<_>>()
        .is_empty()
    {
        panic!("Unbalanced program");
    }
    linked_program
}

#[cfg(test)]
mod tests {
    use self::test::Bencher;
    use super::*;
    use Instruction::*;

    #[bench]
    fn bench_parsing_complicated_program(b: &mut Bencher) {
        let program = "
>>>+[[-]>>[-]++>+>+++++++[<++++>>++<-]++>>+>+>+++++[>++>++++++<<-]+>>>,<++[[>[
->>]<[>>]<<-]<[<]<+>>[>]>[<+>-[[<+>-]>]<[[[-]<]++<-[<+++++++++>[<->-]>>]>>]]<<
]<]<[[<]>[[>]>>[>>]+[<<]<[<]<+>>-]>[>]+[->>]<<<<[[<<]<[<]+<<[+>+<<-[>-->+<<-[>
+<[>>+<<-]]]>[<+>-]<]++>>-->[>]>>[>>]]<<[>>+<[[<]<]>[[<<]<[<]+[-<+>>-[<<+>++>-
[<->[<<+>>-]]]<[>+<-]>]>[>]>]>[>>]>>]<<[>>+>>+>>]<<[->>>>>>>>]<<[>.>>>>>>>]<<[
>->>>>>]<<[>,>>>]<<[>+>]<<[+<<]<]
[input a brainfuck program and its input, separated by an exclamation point.
Daniel B Cristofani (cristofdathevanetdotcom)
http://www.hevanet.com/cristofd/brainfuck/]
".chars()
        .collect::<Vec<char>>();
        b.iter(|| parse(&program));
    }

    #[test]
    fn it_links_jumps() {
        let program = vec![JumpIfZero(0), Add(1), JumpIfNotZero(0)];
        assert_eq!(
            link_jumps(&program),
            vec![JumpIfZero(3), Add(1), JumpIfNotZero(1)]
        );
    }

    #[test]
    #[should_panic]
    fn it_panics_on_unbalanced_jiz() {
        let program = vec![JumpIfZero(0)];
        link_jumps(&program);
    }

    #[test]
    #[should_panic]
    fn it_panics_on_unbalanced_jinz() {
        let program = vec![JumpIfNotZero(0)];
        link_jumps(&program);
    }

    #[test]
    fn it_parses_forward_move() {
        let program = vec!['>'];
        let instructions = vec![Move(1)];
        assert_eq!(parse(&program), instructions);
    }

    #[test]
    fn it_parses_backward_move() {
        let program = vec!['<'];
        let instructions = vec![Move(-1)];
        assert_eq!(parse(&program), instructions);
    }

    #[test]
    fn it_parses_positive_add() {
        let program = vec!['+'];
        let instructions = vec![Add(1)];
        assert_eq!(parse(&program), instructions);
    }

    #[test]
    fn it_parses_negative_add() {
        let program = vec!['-'];
        let instructions = vec![Add(-1)];
        assert_eq!(parse(&program), instructions);
    }

    #[test]
    fn it_parses_print() {
        let program = vec!['.'];
        let instructions = vec![Print];
        assert_eq!(parse(&program), instructions);
    }

    #[test]
    fn it_parses_read() {
        let program = vec![','];
        let instructions = vec![Read];
        assert_eq!(parse(&program), instructions);
    }

    #[test]
    fn it_parses_jump() {
        let program = vec!['[', ']'];
        let instructions = vec![JumpIfZero(2), JumpIfNotZero(1)];
        assert_eq!(parse(&program), instructions);
    }

    #[test]
    fn it_ignores_unknown_token() {
        let program = vec!['!'];
        assert_eq!(parse(&program), vec![]);
    }

    #[test]
    fn it_parses_multiple_instructions() {
        let program = vec!['>', '<', '+', '-', '.', ',', '[', ']'];
        let instructions = vec![
            Move(1),
            Move(-1),
            Add(1),
            Add(-1),
            Print,
            Read,
            JumpIfZero(8),
            JumpIfNotZero(7),
        ];
        assert_eq!(parse(&program), instructions);
    }
}
