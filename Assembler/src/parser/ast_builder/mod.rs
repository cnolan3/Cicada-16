/*
Copyright 2025 Connor Nolan

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
#![allow(dead_code)]

mod arithmetic;
mod bitwise;
mod constants;
mod control_flow;
mod directive;
mod load_store;
mod operand_builders;
mod stack;
mod utility_functions;
mod validators;

use crate::errors::AssemblyError;
use crate::parser::Rule;
use crate::parser::{Directive, Instruction};
use anyhow::Result;
use pest::iterators::{Pair, Pairs};

#[derive(Clone)]
pub struct AstBuilder<'a> {
    line_number: usize,
    rule: Rule,
    pairs: Pairs<'a, Rule>,
}

impl<'a> AstBuilder<'a> {
    pub fn new(pair: Pair<'a, Rule>) -> Self {
        Self {
            line_number: pair.as_span().start_pos().line_col().0,
            rule: pair.as_rule(),
            pairs: pair.into_inner(),
        }
    }

    // Helper to build an Instruction from a pest Pair
    pub fn build_instruction(self) -> Result<Instruction> {
        match self.rule {
            Rule::nop => Ok(Instruction::Nop),
            Rule::halt => Ok(Instruction::Halt),
            Rule::st_2_op => self.build_st(),
            Rule::ld_2_op => self.build_ld(),
            Rule::ldi_2_op => self.build_ldi(),
            Rule::add_sp => self.build_add_sp(),
            Rule::add_2_op => self.build_add_2_op(),
            Rule::addi_2_op => self.build_addi_2_op(),
            Rule::add_1_op => self.build_add_1_op(),
            Rule::addi_1_op => self.build_addi_1_op(),
            Rule::sub_2_op => self.build_sub_2_op(),
            Rule::subi_2_op => self.build_subi_2_op(),
            Rule::sub_1_op => self.build_sub_1_op(),
            Rule::subi_1_op => self.build_subi_1_op(),
            Rule::and_2_op => self.build_and_2_op(),
            Rule::andi_2_op => self.build_andi_2_op(),
            Rule::and_1_op => self.build_and_1_op(),
            Rule::andi_1_op => self.build_andi_1_op(),
            Rule::or_2_op => self.build_or_2_op(),
            Rule::ori_2_op => self.build_ori_2_op(),
            Rule::or_1_op => self.build_or_1_op(),
            Rule::ori_1_op => self.build_ori_1_op(),
            Rule::xor_2_op => self.build_xor_2_op(),
            Rule::xori_2_op => self.build_xori_2_op(),
            Rule::xor_1_op => self.build_xor_1_op(),
            Rule::xori_1_op => self.build_xori_1_op(),
            Rule::cmp_2_op => self.build_cmp_2_op(),
            Rule::cmpi_2_op => self.build_cmpi_2_op(),
            Rule::cmp_1_op => self.build_cmp_1_op(),
            Rule::cmpi_1_op => self.build_cmpi_1_op(),
            Rule::adc_2_op => self.build_adc_2_op(),
            Rule::adci_1_op => self.build_adci_1_op(),
            Rule::sbc_2_op => self.build_sbc_2_op(),
            Rule::sbci_1_op => self.build_sbci_1_op(),
            Rule::ldi_b => self.build_ldi_b(),
            Rule::add_b => self.build_add_b(),
            Rule::sub_b => self.build_sub_b(),
            Rule::and_b => self.build_and_b(),
            Rule::or_b => self.build_or_b(),
            Rule::xor_b => self.build_xor_b(),
            Rule::cmp_b => self.build_cmp_b(),
            Rule::ld_b => self.build_ld_b(),
            Rule::st_b => self.build_st_b(),
            Rule::push_f => Ok(Instruction::PushF),
            Rule::pop_f => Ok(Instruction::PopF),
            Rule::push_op => self.build_push(),
            Rule::pop_op => self.build_pop(),
            Rule::neg => Ok(Instruction::NegAcc),
            Rule::not => Ok(Instruction::NotAcc),
            Rule::swap => Ok(Instruction::SwapAcc),
            Rule::jmp => self.build_jmp(),
            Rule::jr => self.build_jr(),
            Rule::jmp_con => self.build_jcc(),
            Rule::jr_con => self.build_jrcc(),
            Rule::djnz => self.build_djnz(),
            Rule::call => self.build_call(),
            Rule::call_con => self.build_callcc(),
            Rule::syscall => self.build_syscall(),
            Rule::ccf => Ok(Instruction::Ccf),
            Rule::scf => Ok(Instruction::Scf),
            Rule::rcf => Ok(Instruction::Rcf),
            Rule::enter => Ok(Instruction::Enter),
            Rule::leave => Ok(Instruction::Leave),
            Rule::ret => Ok(Instruction::Ret),
            Rule::reti => Ok(Instruction::Reti),
            Rule::ei => Ok(Instruction::Ei),
            Rule::di => Ok(Instruction::Di),
            Rule::inc => self.build_inc(),
            Rule::dec => self.build_dec(),
            Rule::sra => self.build_sra(),
            Rule::shl => self.build_shl(),
            Rule::shr => self.build_shr(),
            Rule::rol => self.build_rol(),
            Rule::ror => self.build_ror(),
            Rule::bit => self.build_bit(),
            Rule::set => self.build_set(),
            Rule::res => self.build_res(),
            Rule::lea => self.build_lea(),
            Rule::call_far => self.build_call_far(),
            Rule::call_far_via => self.build_call_far_via(),
            Rule::jmp_far => self.build_jmp_far(),
            Rule::jmp_far_via => self.build_jmp_far_via(),
            _ => unreachable!("Unknown instruction rule: {:?}", self.rule),
        }
    }

    pub fn build_directive(self) -> Result<Directive> {
        match self.rule {
            Rule::org_directive => self.build_org_directive(),
            Rule::bank_directive => self.build_bank_directive(),
            Rule::byte_directive => self.build_byte_directive(),
            Rule::word_directive => self.build_word_directive(),
            Rule::define_directive => self.build_define_directive(),
            Rule::include_directive => self.build_include_directive(),
            Rule::inc_tiledata_directive => self.build_inc_tiledata_directive(),
            Rule::header_directive_block => self.build_header_directive(),
            Rule::interrupt_directive_block => self.build_interrupt_directive(),
            _ => unreachable!("Unknown directive rule: {:?}", self.rule),
        }
    }
}
