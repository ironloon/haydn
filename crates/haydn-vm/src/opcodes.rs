use crate::types::*;
use crate::HaydnVm;

fn pop_two(stack: &mut Vec<i64>) -> (i64, i64, Option<EdgeCase>) {
    match stack.len() {
        0 => (0, 0, Some(EdgeCase::EmptyStackDefault)),
        1 => {
            let b = stack.pop().unwrap();
            (0, b, Some(EdgeCase::StackUnderflow))
        }
        _ => {
            let b = stack.pop().unwrap();
            let a = stack.pop().unwrap();
            (a, b, None)
        }
    }
}

pub(crate) fn execute_opcode(vm: &mut HaydnVm, opcode: Opcode) -> (Operation, Option<Vec<u8>>, Option<EdgeCase>) {
    match opcode {
        Opcode::Dup => {
            let (val, edge) = match vm.stack.last().copied() {
                Some(v) => (v, None),
                None => (0, Some(EdgeCase::EmptyStackDefault)),
            };
            vm.stack.push(val);
            (Operation::Executed(Opcode::Dup), None, edge)
        }
        Opcode::Swap => {
            let len = vm.stack.len();
            if len >= 2 {
                vm.stack.swap(len - 1, len - 2);
            }
            (Operation::Executed(Opcode::Swap), None, None)
        }
        Opcode::Drop => {
            vm.stack.pop();
            (Operation::Executed(Opcode::Drop), None, None)
        }
        Opcode::Rotate => {
            let len = vm.stack.len();
            if len >= 3 {
                let third = vm.stack.remove(len - 3);
                vm.stack.push(third);
            }
            (Operation::Executed(Opcode::Rotate), None, None)
        }
        Opcode::Add => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            vm.stack.push(a.wrapping_add(b));
            (Operation::Executed(Opcode::Add), None, edge)
        }
        Opcode::Sub => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            vm.stack.push(a.wrapping_sub(b));
            (Operation::Executed(Opcode::Sub), None, edge)
        }
        Opcode::Mul => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            vm.stack.push(a.wrapping_mul(b));
            (Operation::Executed(Opcode::Mul), None, edge)
        }
        Opcode::Div => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            if b == 0 {
                vm.stack.push(0);
                (Operation::Executed(Opcode::Div), None, Some(EdgeCase::DivisionByZero))
            } else {
                vm.stack.push(a.wrapping_div(b));
                (Operation::Executed(Opcode::Div), None, edge)
            }
        }
        Opcode::Mod => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            if b == 0 {
                vm.stack.push(0);
                (Operation::Executed(Opcode::Mod), None, Some(EdgeCase::ModuloByZero))
            } else {
                vm.stack.push(a.wrapping_rem(b));
                (Operation::Executed(Opcode::Mod), None, edge)
            }
        }
        Opcode::Eq => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            vm.stack.push(if a == b { 1 } else { 0 });
            (Operation::Executed(Opcode::Eq), None, edge)
        }
        Opcode::Gt => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            vm.stack.push(if a > b { 1 } else { 0 });
            (Operation::Executed(Opcode::Gt), None, edge)
        }
        Opcode::Lt => {
            let (a, b, edge) = pop_two(&mut vm.stack);
            vm.stack.push(if a < b { 1 } else { 0 });
            (Operation::Executed(Opcode::Lt), None, edge)
        }
        Opcode::Store => {
            let (val, addr, edge) = pop_two(&mut vm.stack);
            if addr < 0 {
                (Operation::Executed(Opcode::Store), None, Some(EdgeCase::NegativeAddress))
            } else {
                if val == 0 {
                    vm.memory.remove(&addr);
                } else {
                    vm.memory.insert(addr, val);
                }
                (Operation::Executed(Opcode::Store), None, edge)
            }
        }
        Opcode::Load => {
            let (addr, edge) = match vm.stack.pop() {
                Some(a) => (a, None),
                None => (0, Some(EdgeCase::EmptyStackDefault)),
            };
            if addr < 0 {
                vm.stack.push(0);
                (Operation::Executed(Opcode::Load), None, Some(EdgeCase::NegativeAddress))
            } else {
                let val = vm.memory.get(&addr).copied().unwrap_or(0);
                vm.stack.push(val);
                (Operation::Executed(Opcode::Load), None, edge)
            }
        }
        Opcode::PrintNum => {
            let val = vm.stack.pop().unwrap_or(0);
            let formatted = format!("{}", val);
            let bytes = formatted.into_bytes();
            vm.output_buffer.extend_from_slice(&bytes);
            (Operation::Executed(Opcode::PrintNum), Some(bytes), None)
        }
        Opcode::PrintChar => {
            match vm.stack.pop() {
                Some(val) => {
                    let byte = val.rem_euclid(256) as u8;
                    vm.output_buffer.push(byte);
                    (Operation::Executed(Opcode::PrintChar), Some(vec![byte]), None)
                }
                None => {
                    // Empty stack: no-op (do not print NUL)
                    (Operation::Executed(Opcode::PrintChar), None, None)
                }
            }
        }
        Opcode::Read => {
            let val = vm.input_buffer.pop_front().map(|b| b as i64).unwrap_or(0);
            vm.stack.push(val);
            (Operation::Executed(Opcode::Read), None, None)
        }
        Opcode::LoopStart | Opcode::LoopEnd => {
            unreachable!("Loop opcodes handled in step()")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(events: &[Event]) -> (HaydnVm, Vec<StepResult>) {
        let mut vm = HaydnVm::new();
        let mut results = Vec::new();
        for &event in events {
            results.extend(vm.process_event(event));
        }
        (vm, results)
    }

    // === DUP ===

    #[test]
    fn test_dup_normal() {
        let (_, results) = run(&[Event::Push(3), Event::Op(Opcode::Dup)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![3, 3]);
        assert!(results.last().unwrap().edge_case.is_none());
    }

    #[test]
    fn test_dup_empty_stack() {
        let (_, results) = run(&[Event::Op(Opcode::Dup)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::EmptyStackDefault));
    }

    // === SWAP ===

    #[test]
    fn test_swap_normal() {
        let (_, results) = run(&[Event::Push(1), Event::Push(2), Event::Op(Opcode::Swap)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![2, 1]);
    }

    #[test]
    fn test_swap_one_element() {
        let (_, results) = run(&[Event::Push(1), Event::Op(Opcode::Swap)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![1]);
    }

    #[test]
    fn test_swap_empty() {
        let (_, results) = run(&[Event::Op(Opcode::Swap)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![]);
    }

    // === DROP ===

    #[test]
    fn test_drop_normal() {
        let (_, results) = run(&[Event::Push(5), Event::Op(Opcode::Drop)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![]);
    }

    #[test]
    fn test_drop_empty() {
        let (_, results) = run(&[Event::Op(Opcode::Drop)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![]);
    }

    // === ROTATE ===

    #[test]
    fn test_rotate_normal() {
        let (_, results) = run(&[
            Event::Push(1), Event::Push(2), Event::Push(3),
            Event::Op(Opcode::Rotate),
        ]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![2, 3, 1]);
    }

    #[test]
    fn test_rotate_two_elements() {
        let (_, results) = run(&[Event::Push(1), Event::Push(2), Event::Op(Opcode::Rotate)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![1, 2]);
    }

    // === ADD ===

    #[test]
    fn test_add_normal() {
        let (_, results) = run(&[Event::Push(3), Event::Push(4), Event::Op(Opcode::Add)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![7]);
    }

    #[test]
    fn test_add_one_element() {
        let (_, results) = run(&[Event::Push(5), Event::Op(Opcode::Add)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![5]);
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::StackUnderflow));
    }

    #[test]
    fn test_add_empty() {
        let (_, results) = run(&[Event::Op(Opcode::Add)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::EmptyStackDefault));
    }

    #[test]
    fn test_add_wrapping() {
        let (_, results) = run(&[Event::Push(i64::MAX), Event::Push(1), Event::Op(Opcode::Add)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![i64::MIN]);
    }

    // === SUB ===

    #[test]
    fn test_sub_normal() {
        let (_, results) = run(&[Event::Push(10), Event::Push(3), Event::Op(Opcode::Sub)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![7]);
    }

    #[test]
    fn test_sub_one_element() {
        let (_, results) = run(&[Event::Push(5), Event::Op(Opcode::Sub)]);
        // a=0, b=5, result = 0 - 5 = -5
        assert_eq!(results.last().unwrap().stack_snapshot, vec![-5]);
    }

    // === MUL ===

    #[test]
    fn test_mul_normal() {
        let (_, results) = run(&[Event::Push(3), Event::Push(4), Event::Op(Opcode::Mul)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![12]);
    }

    #[test]
    fn test_mul_empty() {
        let (_, results) = run(&[Event::Op(Opcode::Mul)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
    }

    // === DIV ===

    #[test]
    fn test_div_normal() {
        let (_, results) = run(&[Event::Push(10), Event::Push(3), Event::Op(Opcode::Div)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![3]);
    }

    #[test]
    fn test_div_by_zero() {
        let (_, results) = run(&[Event::Push(10), Event::Push(0), Event::Op(Opcode::Div)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::DivisionByZero));
    }

    #[test]
    fn test_div_truncates_toward_zero() {
        let (_, results) = run(&[Event::Push(-7), Event::Push(2), Event::Op(Opcode::Div)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![-3]);
    }

    // === MOD ===

    #[test]
    fn test_mod_normal() {
        let (_, results) = run(&[Event::Push(10), Event::Push(3), Event::Op(Opcode::Mod)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![1]);
    }

    #[test]
    fn test_mod_negative_dividend() {
        let (_, results) = run(&[Event::Push(-10), Event::Push(3), Event::Op(Opcode::Mod)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![-1]);
    }

    #[test]
    fn test_mod_by_zero() {
        let (_, results) = run(&[Event::Push(10), Event::Push(0), Event::Op(Opcode::Mod)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::ModuloByZero));
    }

    // === EQ ===

    #[test]
    fn test_eq_equal() {
        let (_, results) = run(&[Event::Push(5), Event::Push(5), Event::Op(Opcode::Eq)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![1]);
    }

    #[test]
    fn test_eq_not_equal() {
        let (_, results) = run(&[Event::Push(5), Event::Push(3), Event::Op(Opcode::Eq)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
    }

    #[test]
    fn test_eq_empty() {
        let (_, results) = run(&[Event::Op(Opcode::Eq)]);
        // 0 == 0 → 1
        assert_eq!(results.last().unwrap().stack_snapshot, vec![1]);
    }

    // === GT ===

    #[test]
    fn test_gt_true() {
        let (_, results) = run(&[Event::Push(5), Event::Push(3), Event::Op(Opcode::Gt)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![1]);
    }

    #[test]
    fn test_gt_false() {
        let (_, results) = run(&[Event::Push(3), Event::Push(5), Event::Op(Opcode::Gt)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
    }

    #[test]
    fn test_gt_empty() {
        let (_, results) = run(&[Event::Op(Opcode::Gt)]);
        // 0 > 0 → 0
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
    }

    // === LT ===

    #[test]
    fn test_lt_true() {
        let (_, results) = run(&[Event::Push(3), Event::Push(5), Event::Op(Opcode::Lt)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![1]);
    }

    #[test]
    fn test_lt_false() {
        let (_, results) = run(&[Event::Push(5), Event::Push(3), Event::Op(Opcode::Lt)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
    }

    // === STORE ===

    #[test]
    fn test_store_normal() {
        let (vm, results) = run(&[Event::Push(42), Event::Push(1), Event::Op(Opcode::Store)]);
        assert_eq!(*vm.memory.get(&1).unwrap(), 42);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![]);
    }

    #[test]
    fn test_store_negative_addr() {
        let (vm, results) = run(&[Event::Push(42), Event::Push(-1), Event::Op(Opcode::Store)]);
        assert!(vm.memory.is_empty());
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::NegativeAddress));
    }

    #[test]
    fn test_store_empty_stack() {
        // Missing operands treated as 0 → store 0 at address 0
        let (vm, _) = run(&[Event::Op(Opcode::Store)]);
        // 0 stored at 0 would be pruned (val==0 removes key)
        assert!(!vm.memory.contains_key(&0));
    }

    // === LOAD ===

    #[test]
    fn test_load_normal() {
        let (_, results) = run(&[
            Event::Push(42), Event::Push(1), Event::Op(Opcode::Store),
            Event::Push(1), Event::Op(Opcode::Load),
        ]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![42]);
    }

    #[test]
    fn test_load_uninitialized() {
        let (_, results) = run(&[Event::Push(99), Event::Op(Opcode::Load)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
    }

    #[test]
    fn test_load_negative_addr() {
        let (_, results) = run(&[Event::Push(-1), Event::Op(Opcode::Load)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::NegativeAddress));
    }

    #[test]
    fn test_load_empty_stack() {
        // Empty stack → load from address 0
        let (_, results) = run(&[Event::Op(Opcode::Load)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
        assert_eq!(results.last().unwrap().edge_case, Some(EdgeCase::EmptyStackDefault));
    }

    // === PRINT_NUM ===

    #[test]
    fn test_print_num_normal() {
        let (vm, results) = run(&[Event::Push(42), Event::Op(Opcode::PrintNum)]);
        assert_eq!(results.last().unwrap().output, Some(b"42".to_vec()));
        assert_eq!(vm.output(), b"42");
    }

    #[test]
    fn test_print_num_empty_stack() {
        let (vm, results) = run(&[Event::Op(Opcode::PrintNum)]);
        assert_eq!(results.last().unwrap().output, Some(b"0".to_vec()));
        assert_eq!(vm.output(), b"0");
    }

    #[test]
    fn test_print_num_negative() {
        let (_, results) = run(&[Event::Push(-123), Event::Op(Opcode::PrintNum)]);
        assert_eq!(results.last().unwrap().output, Some(b"-123".to_vec()));
    }

    // === PRINT_CHAR ===

    #[test]
    fn test_print_char_normal() {
        let (vm, results) = run(&[Event::Push(72), Event::Op(Opcode::PrintChar)]);
        assert_eq!(results.last().unwrap().output, Some(vec![72]));
        assert_eq!(vm.output(), b"H");
    }

    #[test]
    fn test_print_char_mod_256() {
        let (_, results) = run(&[Event::Push(256 + 65), Event::Op(Opcode::PrintChar)]);
        assert_eq!(results.last().unwrap().output, Some(vec![65]));
    }

    #[test]
    fn test_print_char_empty_stack() {
        let (vm, results) = run(&[Event::Op(Opcode::PrintChar)]);
        assert!(results.last().unwrap().output.is_none());
        assert!(vm.output().is_empty());
    }

    #[test]
    fn test_print_char_negative_wraps() {
        // -1 rem_euclid 256 = 255
        let (_, results) = run(&[Event::Push(-1), Event::Op(Opcode::PrintChar)]);
        assert_eq!(results.last().unwrap().output, Some(vec![255]));
    }

    // === READ ===

    #[test]
    fn test_read_with_input() {
        let mut vm = HaydnVm::new();
        vm.provide_input(b"A");
        let results = vm.process_event(Event::Op(Opcode::Read));
        assert_eq!(results.last().unwrap().stack_snapshot, vec![65]);
    }

    #[test]
    fn test_read_empty_input() {
        let (_, results) = run(&[Event::Op(Opcode::Read)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![0]);
    }

    // === Spec example tests ===

    #[test]
    fn test_spec_hello_push_and_print() {
        // Spec §8.1
        let (vm, _) = run(&[
            Event::Push(72), Event::Op(Opcode::PrintChar),
            Event::Push(101), Event::Op(Opcode::PrintChar),
            Event::Push(108), Event::Op(Opcode::PrintChar),
            Event::Push(108), Event::Op(Opcode::PrintChar),
            Event::Push(111), Event::Op(Opcode::PrintChar),
        ]);
        assert_eq!(vm.output(), b"Hello");
    }

    #[test]
    fn test_spec_add_two_numbers() {
        // Spec §8.2: Push 3, Push 4, add, print_num → "7"
        let (vm, _) = run(&[
            Event::Push(3), Event::Push(4), Event::Op(Opcode::Add),
            Event::Op(Opcode::PrintNum),
        ]);
        assert_eq!(vm.output(), b"7");
    }

    // === Additional edge case tests ===

    #[test]
    fn test_sub_operand_order() {
        // Push(10), Push(3), Sub → 10 - 3 = 7, NOT 3 - 10
        let (_, results) = run(&[Event::Push(10), Event::Push(3), Event::Op(Opcode::Sub)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![7]);
    }

    #[test]
    fn test_wrapping_sub() {
        let (_, results) = run(&[Event::Push(i64::MIN), Event::Push(1), Event::Op(Opcode::Sub)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![i64::MAX]);
    }

    #[test]
    fn test_wrapping_mul() {
        let (_, results) = run(&[Event::Push(i64::MAX), Event::Push(2), Event::Op(Opcode::Mul)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![-2]);
    }

    #[test]
    fn test_div_min_by_neg_one() {
        // i64::MIN / -1 would overflow — wrapping_div handles this
        let (_, results) = run(&[Event::Push(i64::MIN), Event::Push(-1), Event::Op(Opcode::Div)]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![i64::MIN]);
    }

    #[test]
    fn test_multiple_operations_sequence() {
        // Push 5, Push 3, add (=8), Push 2, mul (=16), print_num → "16"
        let (vm, _) = run(&[
            Event::Push(5), Event::Push(3), Event::Op(Opcode::Add),
            Event::Push(2), Event::Op(Opcode::Mul),
            Event::Op(Opcode::PrintNum),
        ]);
        assert_eq!(vm.output(), b"16");
    }

    #[test]
    fn test_memory_roundtrip() {
        // Store 99 at address 5, load from address 5
        let (_, results) = run(&[
            Event::Push(99), Event::Push(5), Event::Op(Opcode::Store),
            Event::Push(5), Event::Op(Opcode::Load),
        ]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![99]);
    }

    #[test]
    fn test_dup_swap_drop_sequence() {
        // Push 7, dup → [7,7], drop → [7]
        let (_, results) = run(&[
            Event::Push(7), Event::Op(Opcode::Dup), Event::Op(Opcode::Drop),
        ]);
        assert_eq!(results.last().unwrap().stack_snapshot, vec![7]);
    }

    #[test]
    fn test_read_multiple_bytes() {
        let mut vm = HaydnVm::new();
        vm.provide_input(b"Hi");
        let r1 = vm.process_event(Event::Op(Opcode::Read));
        let r2 = vm.process_event(Event::Op(Opcode::Read));
        let r3 = vm.process_event(Event::Op(Opcode::Read));
        assert_eq!(r1.last().unwrap().stack_snapshot, vec![72]);  // 'H'
        assert_eq!(r2.last().unwrap().stack_snapshot, vec![72, 105]);  // 'i'
        assert_eq!(r3.last().unwrap().stack_snapshot, vec![72, 105, 0]);  // no input → 0
    }
}
