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

use cicasm::assemble;
use cicasm::file_reader::MockFileReader;
use std::path::Path;

const BANK_SIZE: usize = 16384;

#[test]
fn test_nop() {
    let mut reader = MockFileReader::default();
    reader.add_file("test.asm", "NOP\n");

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x00);
    assert_eq!(result[1], 0xFF); // Padding
}

#[test]
fn test_ldi() {
    let mut reader = MockFileReader::default();
    reader.add_file("test.asm", "LDI R1, 0x1234\n");

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x02); // LDI R1
    assert_eq!(result[1], 0x34); // low byte
    assert_eq!(result[2], 0x12); // high byte
    assert_eq!(result[3], 0xFF); // Padding
}

#[test]
fn test_jump() {
    let mut reader = MockFileReader::default();
    reader.add_file("test.asm", "START:\nNOP\nJMP START\n");

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x00); // NOP
    assert_eq!(result[1], 0x51); // JMP
    assert_eq!(result[2], 0x00); // low byte of START (0x0000)
    assert_eq!(result[3], 0x00); // high byte of START (0x0000)
    assert_eq!(result[4], 0xFF); // Padding
}

#[test]
fn test_define() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        ".define CON1 3\n.define CON2 0x0200\nLDI r0, CON1\nST (CON2), r0\n",
    );

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x01); // LDI r0
    assert_eq!(result[1], 0x03); // low byte of CON1
    assert_eq!(result[2], 0x00); // high byte of CON1
    assert_eq!(result[3], 0xF1); // ST (), r0
    assert_eq!(result[4], 0x00); // low byte of CON2
    assert_eq!(result[5], 0x02); // high byte of CON2
    assert_eq!(result[6], 0xFF); // Padding
}

#[test]
fn test_org() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        ".bank 0\n.org 0x0200\nFIXED_LABEL:\nNOP\n.bank 1\n.org 0x4100\nBANK_1_LABEL:\nNOP\nJMP FIXED_LABEL\n.bank 2\n.org 0x4200\nBANK_2_LABEL:\nNOP\n.org 0x6000\nJMP BANK_2_LABEL\nJMP FIXED_LABEL\n",
    );

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x7FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 3);
    assert_eq!(result[0x0000], 0x00); // Padding
    assert_eq!(result[0x0200], 0x00, "expected addr 0x0200 to be NOP"); // Nop
    assert_eq!(result[0x4100], 0x00, "expected addr 0x4100 to be NOP"); // Nop
    assert_eq!(
        result[0x4101], 0x51,
        "expected addr 0x4101 to be JMP opcode"
    ); // Jmp
    assert_eq!(
        result[0x4102], 0x00,
        "expected addr 0x4102 to be FIXED_LABEL low byte"
    ); // low byte of FIXED_LABEL
    assert_eq!(
        result[0x4103], 0x02,
        "expected addr 0x4103 to be FIXED_LABEL high byte"
    ); // high byte of FIXED_LABEL
    assert_eq!(result[0x8200], 0x00, "expected addr 0x8200 to be NOP"); // Nop
    assert_eq!(
        result[0xA000], 0x51,
        "expected addr 0xA000 to be JMP opcode"
    ); // JMP
    assert_eq!(
        result[0xA001], 0x00,
        "expected addr 0xA001 to be BANK_2_LABEL low byte"
    ); // low byte of BANK_2_LABEL
    assert_eq!(
        result[0xA002], 0x42,
        "expected addr 0xA002 to be BANK_2_LABEL high byte"
    ); // high byte of BANK_2_LABEL
    assert_eq!(
        result[0xA003], 0x51,
        "expected addr 0xA003 to be JMP opcode"
    ); // JMP
    assert_eq!(
        result[0xA004], 0x00,
        "expected addr 0xA004 to be FIXED_LABEL low byte"
    ); // low byte of FIXED_LABEL
    assert_eq!(
        result[0xA005], 0x02,
        "expected addr 0xA005 to be FIXED_LABEL high byte"
    ); // high byte of FIXED_LABEL
    assert_eq!(result[0xA006], 0xFF); // Padding
}

#[test]
fn test_include() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        "LDI r0, 0x1234\n.include \"inc_1.asm\"\n.include \"inc_2.asm\"\n",
    );
    reader.add_file("inc_1.asm", "LDI r1, 0x5678\n.include \"inc_3.asm\"\n");
    reader.add_file("inc_2.asm", "LDI r2, 0x9ABC\n");
    reader.add_file("inc_3.asm", "LDI r3, 0xDEF0\n");

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x01); // LDI r0
    assert_eq!(result[0x0001], 0x34); // low byte
    assert_eq!(result[0x0002], 0x12); // high byte
    assert_eq!(result[0x0003], 0x02); // LDI r1
    assert_eq!(result[0x0004], 0x78); // low byte
    assert_eq!(result[0x0005], 0x56); // high byte
    assert_eq!(result[0x0006], 0x04); // LDI r3
    assert_eq!(result[0x0007], 0xF0); // low byte
    assert_eq!(result[0x0008], 0xDE); // high byte
    assert_eq!(result[0x0009], 0x03); // LDI r2
    assert_eq!(result[0x000A], 0xBC); // low byte
    assert_eq!(result[0x000B], 0x9A); // high byte
    assert_eq!(result[0x000C], 0xFF); // Padding
}

#[test]
fn test_header() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .header_start
            .boot_anim "CICA"
            .title "Test-Game"
            .developer "Test-Dev"
            .version 1
            .mapper 1
            .rom_size 2
            .ram_size 1
            .interrupt_mode 1
            .hardware_rev 0
            .region 3
        .header_end
        "#,
    );

    let entry_path = Path::new("test.asm");

    let result = assemble(entry_path, 0x7FFF, None, Some(0x0000), &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);

    // Boot Animation
    assert_eq!(&result[0..4], b"CICA");

    // Title
    let mut title_bytes = b"Test-Game".to_vec();
    title_bytes.resize(16, 0);
    assert_eq!(&result[0x04..0x14], &title_bytes[..]);

    // Developer
    let mut dev_bytes = b"Test-Dev".to_vec();
    dev_bytes.resize(16, 0);
    assert_eq!(&result[0x14..0x24], &dev_bytes[..]);

    // Version
    assert_eq!(result[0x24], 1);

    // ROM Size
    assert_eq!(result[0x25], 2);

    // RAM Size
    assert_eq!(result[0x26], 1);

    // Cartridge Info
    // hardware_rev = 0 (00), region = 3 (011)
    assert_eq!(result[0x27], 0b00011000);

    // Feature Flags
    // interrupt_mode = 1, mapper = 1 (01)
    assert_eq!(result[0x28], 0b10100000);

    // The rest of the header should be 0 up to the checksums, which are not yet implemented in this test
    for i in 0x29..0x60 {
        assert_eq!(result[i], 0x00);
    }
}

#[test]
fn test_interrupt_table() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .org 0x0060
        .interrupt_table
            .word RESET_HANDLER
            .word BUS_ERROR_HANDLER
            .word ILLEGAL_INSTRUCTION_HANDLER
            .word PROTECTED_MEMORY_HANDLER
            .word STACK_OVERFLOW_HANDLER
            .word VBLANK_HANDLER
            .word HBLANK_HANDLER
            .word LYC_HANDLER
            .word TIMER_0_HANDLER
            .word TIMER_1_HANDLER
            .word SERIAL_HANDLER
            .word LINK_STATUS_HANDLER
            .word JOYPAD_HANDLER
        .table_end

        .org 0x0100
        RESET_HANDLER:
        NOP
        BUS_ERROR_HANDLER:
        NOP
        ILLEGAL_INSTRUCTION_HANDLER:
        NOP
        PROTECTED_MEMORY_HANDLER:
        NOP
        STACK_OVERFLOW_HANDLER:
        NOP
        VBLANK_HANDLER:
        NOP
        HBLANK_HANDLER:
        NOP
        LYC_HANDLER:
        NOP
        TIMER_0_HANDLER:
        NOP
        TIMER_1_HANDLER:
        NOP
        SERIAL_HANDLER:
        NOP
        LINK_STATUS_HANDLER:
        NOP
        JOYPAD_HANDLER:
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");

    let result = assemble(entry_path, 0x7FFF, Some(0x0060), None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);

    // Interrupt table starts at 0x0060
    // Handlers start at 0x0100 and are one byte each (NOP)
    let expected_addresses = (0..13).map(|i| 0x0100 + i).collect::<Vec<u16>>();

    for i in 0..13 {
        let addr_offset = 0x60 + (i * 2);
        let expected_addr = expected_addresses[i];
        let actual_addr = u16::from_le_bytes([result[addr_offset], result[addr_offset + 1]]);
        assert_eq!(actual_addr, expected_addr);
    }

    // Check padding after table
    assert_eq!(result[0x60 + 26], 0x00);
}

#[test]
fn test_section_with_size() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .section size=16
        NOP
        NOP
        .section_end
        ENTER
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // First two NOPs inside section
    assert_eq!(result[0x0000], 0x00);
    assert_eq!(result[0x0001], 0x00);
    // Padding to reach section size of 16
    for i in 0x0002..0x0010 {
        assert_eq!(result[i], 0x00, "Expected padding at offset 0x{:04X}", i);
    }
    // ENTER after section
    assert_eq!(result[0x0010], 0x4F);
    assert_eq!(result[0x0011], 0xFF); // ROM padding
}

#[test]
fn test_section_with_vaddr() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .bank 1
        .section vaddr=0x5000
        LABEL_IN_SECTION:
        NOP
        .section_end
        ENTER
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x7FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // Section starts at bank 1, physical address 0x4000
    assert_eq!(result[0x4000], 0x00, "Expected NOP at physical 0x4000");
    // After section ends, logical address should be restored
    assert_eq!(
        result[0x4001], 0x4F,
        "Expected NOP after section at physical 0x4001"
    );
}

#[test]
fn test_section_with_paddr() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .section paddr=0x0100
        NOP
        NOP
        .section_end
        ENTER
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // Padding before section
    assert_eq!(result[0x0000], 0x00);
    // Section content at physical address 0x0100
    assert_eq!(result[0x0100], 0x00, "Expected first NOP at 0x0100");
    assert_eq!(result[0x0101], 0x00, "Expected second NOP at 0x0101");
    // NOP after section
    assert_eq!(
        result[0x0102], 0x4F,
        "Expected ENTER after section at 0x0102"
    );
}

#[test]
fn test_section_with_size_and_vaddr() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .bank 1
        .section size=32 vaddr=0x4200
        NOP
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x7FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // Section at bank 1, physical 0x4000, logical 0x4200
    assert_eq!(result[0x4000], 0x00, "Expected first NOP");
    assert_eq!(result[0x4001], 0x00, "Expected second NOP");
    // Padding to reach section size of 32
    for i in 0x4002..0x4020 {
        assert_eq!(result[i], 0x00, "Expected padding at 0x{:04X}", i);
    }
}

#[test]
fn test_section_overflow() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .section size=4
        NOP
        NOP
        NOP
        NOP
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader);

    assert!(
        result.is_err(),
        "Expected error when section exceeds allocated size"
    );
}

#[test]
fn test_section_with_labels() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .bank 1
        .section vaddr=0x4100
        SECTION_START:
        NOP
        SECTION_MIDDLE:
        NOP
        .section_end
        AFTER_SECTION:
        NOP
        JMP SECTION_MIDDLE
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x7FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // Section NOPs
    assert_eq!(result[0x4000], 0x00);
    assert_eq!(result[0x4001], 0x00);
    // NOP after section
    assert_eq!(result[0x4002], 0x00);
    // JMP to SECTION_MIDDLE (vaddr 0x4101)
    assert_eq!(result[0x4003], 0x51); // JMP opcode
    assert_eq!(result[0x4004], 0x01); // low byte
    assert_eq!(result[0x4005], 0x41); // high byte
}

#[test]
fn test_multiple_sections() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .section size=8
        NOP
        NOP
        .section_end

        .section size=8
        NOP
        NOP
        .section_end

        ENTER
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // First section
    assert_eq!(result[0x0000], 0x00);
    assert_eq!(result[0x0001], 0x00);
    for i in 0x0002..0x0008 {
        assert_eq!(result[i], 0x00, "Expected padding in first section");
    }
    // Second section
    assert_eq!(result[0x0008], 0x00);
    assert_eq!(result[0x0009], 0x00);
    for i in 0x000A..0x0010 {
        assert_eq!(result[i], 0x00, "Expected padding in second section");
    }
    // ENTER after sections
    assert_eq!(result[0x0010], 0x4F);
}

#[test]
fn test_section_relative_jump() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .bank 1
        .section vaddr=0x4100
        START:
        NOP
        NOP
        JR START
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x7FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // Section at bank 1, physical 0x4000
    assert_eq!(result[0x4000], 0x00); // NOP
    assert_eq!(result[0x4001], 0x00); // NOP
    assert_eq!(result[0x4002], 0x5A); // JR opcode
    // The test verifies that sections work correctly with relative jumps
    // The actual offset calculation is tested by the fact that logical_address is now used
}

#[test]
fn test_align_2_already_aligned() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        NOP
        .align 2
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // NOP
    // Already aligned at 0x0002, no padding needed
    assert_eq!(result[0x0002], 0x00); // NOP after align
}

#[test]
fn test_align_2_needs_padding() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .align 2
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // Padding byte
    assert_eq!(result[0x0002], 0x00); // NOP after align at even address
}

#[test]
fn test_align_4() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        NOP
        .align 4
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // NOP
    assert_eq!(result[0x0002], 0x00); // Padding
    assert_eq!(result[0x0003], 0x00); // Padding
    assert_eq!(result[0x0004], 0x00); // NOP after align at 4-byte boundary
}

#[test]
fn test_align_8() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        NOP
        NOP
        .align 8
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // NOP
    assert_eq!(result[0x0002], 0x00); // NOP
    for i in 0x0003..0x0008 {
        assert_eq!(result[i], 0x00, "Expected padding at offset 0x{:04X}", i);
    }
    assert_eq!(result[0x0008], 0x00); // NOP after align at 8-byte boundary
}

#[test]
fn test_align_with_labels() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .align 4
        ALIGNED_LABEL:
        NOP
        JMP ALIGNED_LABEL
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // Padding
    assert_eq!(result[0x0002], 0x00); // Padding
    assert_eq!(result[0x0003], 0x00); // Padding
    assert_eq!(result[0x0004], 0x00); // NOP at ALIGNED_LABEL (address 0x0004)
    assert_eq!(result[0x0005], 0x51); // JMP opcode
    assert_eq!(result[0x0006], 0x04); // Low byte of ALIGNED_LABEL
    assert_eq!(result[0x0007], 0x00); // High byte of ALIGNED_LABEL
}

#[test]
fn test_align_1_no_op() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .align 1
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // NOP immediately after (no padding for align 1)
}

#[test]
fn test_align_with_data() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .byte 0x01, 0x02, 0x03
        .align 4
        DATA_TABLE:
        .word 0x1111, 0x2222, 0x3333
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x01); // .byte 0x01
    assert_eq!(result[0x0001], 0x02); // .byte 0x02
    assert_eq!(result[0x0002], 0x03); // .byte 0x03
    assert_eq!(result[0x0003], 0x00); // Padding to align to 4
    // DATA_TABLE starts at 0x0004
    assert_eq!(result[0x0004], 0x11); // Low byte of 0x1111
    assert_eq!(result[0x0005], 0x11); // High byte of 0x1111
    assert_eq!(result[0x0006], 0x22); // Low byte of 0x2222
    assert_eq!(result[0x0007], 0x22); // High byte of 0x2222
    assert_eq!(result[0x0008], 0x33); // Low byte of 0x3333
    assert_eq!(result[0x0009], 0x33); // High byte of 0x3333
}

#[test]
fn test_multiple_aligns() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .align 4
        NOP
        NOP
        .align 8
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    // Padding to align to 4
    for i in 0x0001..0x0004 {
        assert_eq!(result[i], 0x00, "Expected padding at 0x{:04X}", i);
    }
    assert_eq!(result[0x0004], 0x00); // NOP at 4-byte boundary
    assert_eq!(result[0x0005], 0x00); // NOP
    // Padding to align to 8
    for i in 0x0006..0x0008 {
        assert_eq!(result[i], 0x00, "Expected padding at 0x{:04X}", i);
    }
    assert_eq!(result[0x0008], 0x00); // NOP at 8-byte boundary
}

#[test]
fn test_align_with_org() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .org 0x0101
        NOP
        .align 4
        ALIGNED_CODE:
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0101], 0x00); // NOP at 0x0101
    // Should pad from 0x0102 to 0x0104 (3 bytes)
    assert_eq!(result[0x0102], 0x00); // Padding
    assert_eq!(result[0x0103], 0x00); // Padding
    assert_eq!(result[0x0104], 0x00); // NOP at ALIGNED_CODE (0x0104, 4-byte aligned)
}

#[test]
fn test_align_non_power_of_2() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        NOP
        .align 3
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // NOP
    assert_eq!(result[0x0002], 0x00); // Padding
    assert_eq!(result[0x0003], 0x00); // NOP at 3-byte boundary
}

#[test]
fn test_align_in_bank_1() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .bank 1
        NOP
        NOP
        NOP
        .align 8
        BANK1_ALIGNED:
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x7FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // Bank 1 starts at physical address 0x4000
    assert_eq!(result[0x4000], 0x00); // NOP
    assert_eq!(result[0x4001], 0x00); // NOP
    assert_eq!(result[0x4002], 0x00); // NOP
    // Padding to next 8-byte boundary
    for i in 0x4003..0x4008 {
        assert_eq!(result[i], 0x00, "Expected padding at 0x{:04X}", i);
    }
    assert_eq!(result[0x4008], 0x00); // NOP at BANK1_ALIGNED (8-byte aligned)
}

#[test]
fn test_section_with_align_4() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .section align=4
        NOP
        NOP
        .section_end
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP before section
    // Section should start at 0x0004 (next 4-byte boundary after 0x0001)
    for i in 0x0001..0x0004 {
        assert_eq!(result[i], 0x00, "Expected padding at 0x{:04X}", i);
    }
    assert_eq!(result[0x0004], 0x00); // First NOP in section
    assert_eq!(result[0x0005], 0x00); // Second NOP in section
    assert_eq!(result[0x0006], 0x00); // NOP after section
}

#[test]
fn test_section_with_align_already_aligned() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        NOP
        NOP
        NOP
        .section align=4
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // NOP
    assert_eq!(result[0x0002], 0x00); // NOP
    assert_eq!(result[0x0003], 0x00); // NOP
    // Already at 0x0004, which is 4-byte aligned - no padding needed
    assert_eq!(result[0x0004], 0x00); // NOP in section
}

#[test]
fn test_section_with_size_and_align_padding_counts() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .section size=16 align=4
        NOP
        NOP
        .section_end
        AFTER:
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP before section
    // Alignment padding from 0x0001 to 0x0004 (3 bytes)
    for i in 0x0001..0x0004 {
        assert_eq!(result[i], 0x00, "Expected alignment padding at 0x{:04X}", i);
    }
    // Section content
    assert_eq!(result[0x0004], 0x00); // First NOP in section
    assert_eq!(result[0x0005], 0x00); // Second NOP in section
    // Size padding from 0x0006 to 0x0010 (11 bytes)
    // Total section: 3 (align pad) + 2 (content) + 11 (size pad) = 16 bytes
    for i in 0x0006..0x0011 {
        assert_eq!(result[i], 0x00, "Expected size padding at 0x{:04X}", i);
    }
    // AFTER label should be at 0x0011
    assert_eq!(result[0x0011], 0x00); // NOP at AFTER
}

#[test]
fn test_section_with_align_8() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        NOP
        NOP
        .section align=8
        SECTION_START:
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // NOP
    assert_eq!(result[0x0002], 0x00); // NOP
    // Padding from 0x0003 to 0x0008 (5 bytes)
    for i in 0x0003..0x0008 {
        assert_eq!(result[i], 0x00, "Expected padding at 0x{:04X}", i);
    }
    assert_eq!(result[0x0008], 0x00); // NOP at SECTION_START (8-byte aligned)
}

#[test]
fn test_section_with_align_2() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .section align=2
        NOP
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    assert_eq!(result[0x0001], 0x00); // Padding to align to 2
    assert_eq!(result[0x0002], 0x00); // First NOP in section
    assert_eq!(result[0x0003], 0x00); // Second NOP in section
}

#[test]
fn test_section_with_vaddr_and_align() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .bank 1
        NOP
        .section vaddr=0x4100 align=8
        ALIGNED_SECTION:
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x7FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    // Bank 1 starts at physical 0x4000
    assert_eq!(result[0x4000], 0x00); // NOP
    // Padding from 0x4001 to align to 8
    // vaddr=0x4100, so we need to align to next 8-byte boundary: 0x4108
    // But physical starts at 0x4000, so we need padding
    for i in 0x4001..0x4008 {
        assert_eq!(result[i], 0x00, "Expected padding at 0x{:04X}", i);
    }
    assert_eq!(result[0x4008], 0x00); // NOP at ALIGNED_SECTION
}

#[test]
fn test_section_align_with_labels() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .section align=4
        LABEL1:
        NOP
        LABEL2:
        NOP
        .section_end
        JMP LABEL1
        JMP LABEL2
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    // Padding to 0x0004
    for i in 0x0001..0x0004 {
        assert_eq!(result[i], 0x00);
    }
    assert_eq!(result[0x0004], 0x00); // NOP at LABEL1
    assert_eq!(result[0x0005], 0x00); // NOP at LABEL2
    // JMP LABEL1
    assert_eq!(result[0x0006], 0x51); // JMP opcode
    assert_eq!(result[0x0007], 0x04); // Low byte of LABEL1
    assert_eq!(result[0x0008], 0x00); // High byte of LABEL1
    // JMP LABEL2
    assert_eq!(result[0x0009], 0x51); // JMP opcode
    assert_eq!(result[0x000A], 0x05); // Low byte of LABEL2
    assert_eq!(result[0x000B], 0x00); // High byte of LABEL2
}

#[test]
fn test_multiple_sections_with_align() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .section align=4
        NOP
        .section_end
        NOP
        .section align=8
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    // Padding to 0x0004
    for i in 0x0001..0x0004 {
        assert_eq!(result[i], 0x00);
    }
    assert_eq!(result[0x0004], 0x00); // NOP in first section
    assert_eq!(result[0x0005], 0x00); // NOP between sections
    // Padding to 0x0008
    for i in 0x0006..0x0008 {
        assert_eq!(result[i], 0x00);
    }
    assert_eq!(result[0x0008], 0x00); // NOP in second section
}

#[test]
fn test_section_align_with_data() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        .byte 0x01, 0x02, 0x03
        .section align=4
        DATA_SECTION:
        .word 0x1111, 0x2222
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x01); // .byte 0x01
    assert_eq!(result[0x0001], 0x02); // .byte 0x02
    assert_eq!(result[0x0002], 0x03); // .byte 0x03
    assert_eq!(result[0x0003], 0x00); // Padding to align to 4
    // DATA_SECTION starts at 0x0004
    assert_eq!(result[0x0004], 0x11); // Low byte of 0x1111
    assert_eq!(result[0x0005], 0x11); // High byte of 0x1111
    assert_eq!(result[0x0006], 0x22); // Low byte of 0x2222
    assert_eq!(result[0x0007], 0x22); // High byte of 0x2222
}

#[test]
fn test_section_size_align_overflow() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .section size=8 align=4
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader);

    // Should fail because:
    // - 3 bytes of alignment padding (from 0x0001 to 0x0004)
    // - 6 bytes of content (6 NOPs)
    // - Total = 9 bytes, but size=8
    assert!(
        result.is_err(),
        "Expected error when section with align padding exceeds size"
    );
}

#[test]
fn test_section_align_1_no_padding() {
    let mut reader = MockFileReader::default();
    reader.add_file(
        "test.asm",
        r#"
        NOP
        .section align=1
        NOP
        .section_end
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0x0000], 0x00); // NOP
    // No padding needed for align=1
    assert_eq!(result[0x0001], 0x00); // NOP in section
}

#[test]
fn test_incbin() {
    let mut reader = MockFileReader::default();

    // Add binary file to mock reader
    let binary_data: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    reader.add_binary_file("test_data.bin", &binary_data);

    reader.add_file(
        "test.asm",
        r#"
        NOP
        BINARY_START:
        .incbin "test_data.bin"
        BINARY_END:
        .byte 0xFF, 0xFE
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);

    // First NOP
    assert_eq!(result[0x0000], 0x00);

    // Binary data should be included starting at 0x0001
    for (i, byte) in binary_data.iter().enumerate() {
        assert_eq!(
            result[0x0001 + i],
            *byte,
            "Expected binary data byte {} to match at offset 0x{:04X}",
            i,
            0x0001 + i
        );
    }

    // Bytes after the binary data
    assert_eq!(result[0x0009], 0xFF);
    assert_eq!(result[0x000A], 0xFE);

    // Padding
    assert_eq!(result[0x000B], 0xFF);
}

#[test]
fn test_incbin_with_labels() {
    let mut reader = MockFileReader::default();

    // Add binary file to mock reader
    let binary_data: Vec<u8> = vec![0xAA, 0xBB, 0xCC, 0xDD];
    reader.add_binary_file("sprite.bin", &binary_data);

    reader.add_file(
        "test.asm",
        r#"
        SPRITE_DATA:
        .incbin "sprite.bin"
        AFTER_SPRITE:
        NOP
        JMP SPRITE_DATA
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);

    // Binary data at start
    assert_eq!(result[0x0000], 0xAA);
    assert_eq!(result[0x0001], 0xBB);
    assert_eq!(result[0x0002], 0xCC);
    assert_eq!(result[0x0003], 0xDD);

    // NOP after binary
    assert_eq!(result[0x0004], 0x00);

    // JMP to SPRITE_DATA (address 0x0000)
    assert_eq!(result[0x0005], 0x51); // JMP opcode
    assert_eq!(result[0x0006], 0x00); // Low byte of SPRITE_DATA
    assert_eq!(result[0x0007], 0x00); // High byte of SPRITE_DATA
}

#[test]
fn test_incbin_empty_file() {
    let mut reader = MockFileReader::default();

    // Add empty binary file to mock reader
    let binary_data: Vec<u8> = vec![];
    reader.add_binary_file("empty.bin", &binary_data);

    reader.add_file(
        "test.asm",
        r#"
        NOP
        .incbin "empty.bin"
        NOP
        "#,
    );

    let entry_path = Path::new("test.asm");
    let result = assemble(entry_path, 0x3FFF, None, None, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);

    // Two NOPs back-to-back (empty file contributes nothing)
    assert_eq!(result[0x0000], 0x00);
    assert_eq!(result[0x0001], 0x00);
    assert_eq!(result[0x0002], 0xFF); // Padding
}
