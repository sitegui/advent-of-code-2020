use crate::data::{Data, ParseBytes, TryFromBytes};
use crate::parser::Parser;
use std::collections::BTreeSet;

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Acc(i16),
    Jmp(i16),
    Nop(i16),
}

#[derive(Debug, Copy, Clone, Default)]
struct State {
    next_instruction: i16,
    accumulator: i16,
}

#[derive(Debug)]
enum ExecutionResult {
    Finished {
        final_state: State,
    },
    Loop {
        /// The state just before the execute of a previously-executed instruction
        final_state: State,
        executed_instructions: BTreeSet<i16>,
    },
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(8);

    let program: Vec<Instruction> = data.lines().map(|line| line.parse_bytes()).collect();

    // Run the program until it loops, in order to answer part 1
    // At the same time, collect all the "fixable" instructions in the cycle
    let mut fixable = Vec::new();
    let result = run_program(
        &program,
        State::default(),
        &BTreeSet::new(),
        |instruction, state| {
            if instruction.can_be_fixed() {
                fixable.push((instruction, state));
            }
        },
    );
    let (part_1, doomed_instructions) = match result {
        ExecutionResult::Loop {
            final_state,
            executed_instructions,
        } => (final_state.accumulator, executed_instructions),
        _ => unreachable!(),
    };

    // Find the instruction that, when fixed, will allow the program to finish.
    // For that, the fixable instructions will be tested one by one, in reverse order
    let mut part_2 = None;
    for (instruction, mut state) in fixable.into_iter().rev() {
        // Execute the fixed instruction, instead of the original
        instruction.fixed().execute(&mut state);

        // Check if the execution now finishes
        let result = run_program(&program, state, &doomed_instructions, |_, _| {});
        if let ExecutionResult::Finished { final_state } = result {
            part_2 = Some(final_state.accumulator as i64);
            break;
        }
    }

    (part_1 as i64, part_2.unwrap())
}

fn run_program<F: FnMut(Instruction, State)>(
    program: &[Instruction],
    initial_state: State,
    already_executed: &BTreeSet<i16>,
    mut before_execute: F,
) -> ExecutionResult {
    let mut state = initial_state;
    let mut executed_instructions = already_executed.clone();

    loop {
        if !executed_instructions.insert(state.next_instruction) {
            return ExecutionResult::Loop {
                final_state: state,
                executed_instructions,
            };
        }

        match program.get(state.next_instruction as usize) {
            None => return ExecutionResult::Finished { final_state: state },
            Some(&instruction) => {
                before_execute(instruction, state);
                instruction.execute(&mut state);
            }
        }
    }
}

impl TryFromBytes for Instruction {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut parser = Parser::new(bytes);
        let operation = parser.try_consume_words(1)?;
        let amount: i16 = parser.into_inner().try_parse_bytes()?;
        match operation {
            b"nop" => Some(Instruction::Nop(amount)),
            b"acc" => Some(Instruction::Acc(amount)),
            b"jmp" => Some(Instruction::Jmp(amount)),
            _ => None,
        }
    }
}

impl Instruction {
    fn can_be_fixed(self) -> bool {
        match self {
            Instruction::Acc(_) => false,
            Instruction::Jmp(_) | Instruction::Nop(_) => true,
        }
    }

    fn fixed(self) -> Self {
        match self {
            Instruction::Jmp(n) => Instruction::Nop(n),
            Instruction::Nop(n) => Instruction::Jmp(n),
            _ => unreachable!(),
        }
    }

    fn execute(self, state: &mut State) {
        match self {
            Instruction::Acc(n) => {
                state.accumulator += n;
                state.next_instruction += 1;
            }
            Instruction::Jmp(n) => {
                state.next_instruction += n;
            }
            Instruction::Nop(_) => {
                state.next_instruction += 1;
            }
        }
    }
}
