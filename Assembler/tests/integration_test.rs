use casm::assemble;
use casm::file_reader::MockFileReader;
use std::fs;
use std::path::Path;

const BANK_SIZE: usize = 16384;

#[test]
fn test_nop() {
    let mut reader = MockFileReader::default();
    reader.add_file("test.asm", "NOP\n");

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x0000, 0x3FFF, &reader).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x00);
    assert_eq!(result[1], 0xFF); // Padding
}

#[test]
fn test_ldi() {
    let mut reader = MockFileReader::default();
    reader.add_file("test.asm", "LDI R1, 0x1234\n");

    let entry_path = Path::new("test.asm");

    let result = assemble(&entry_path, 0x0000, 0x3FFF, &reader).unwrap();

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

    let result = assemble(&entry_path, 0x0000, 0x3FFF, &reader).unwrap();

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

    let result = assemble(&entry_path, 0x0000, 0x3FFF, &reader).unwrap();

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

    let result = assemble(&entry_path, 0x0000, 0x7FFF, &reader).unwrap();

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

    let result = assemble(&entry_path, 0x0000, 0x3FFF, &reader).unwrap();

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
