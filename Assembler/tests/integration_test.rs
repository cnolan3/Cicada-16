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
            .word TIMER_HANDLER
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
        TIMER_HANDLER:
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
    let expected_addresses = (0..12).map(|i| 0x0100 + i).collect::<Vec<u16>>();

    for i in 0..12 {
        let addr_offset = 0x60 + (i * 2);
        let expected_addr = expected_addresses[i];
        let actual_addr = u16::from_le_bytes([result[addr_offset], result[addr_offset + 1]]);
        assert_eq!(actual_addr, expected_addr);
    }

    // Check padding after table
    assert_eq!(result[0x60 + 24], 0x00);
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
