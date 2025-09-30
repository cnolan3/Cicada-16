use casm::assemble;

const BANK_SIZE: usize = 16384;

#[test]
fn test_nop() {
    let source = "NOP\n";
    let result = assemble(source, 0x0000, 0x3FFF).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x00);
    assert_eq!(result[1], 0xFF); // Padding
}

#[test]
fn test_ldi() {
    let source = "LDI R1, 0x1234\n";
    let result = assemble(source, 0x0000, 0x3FFF).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x02); // LDI R1
    assert_eq!(result[1], 0x34); // low byte
    assert_eq!(result[2], 0x12); // high byte
    assert_eq!(result[3], 0xFF); // Padding
}

#[test]
fn test_jump() {
    let source = "START:\nNOP\nJMP START\n";
    let result = assemble(source, 0x0000, 0x3FFF).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x00); // NOP
    assert_eq!(result[1], 0x51); // JMP
    assert_eq!(result[2], 0x00); // low byte of START (0x0000)
    assert_eq!(result[3], 0x00); // high byte of START (0x0000)
    assert_eq!(result[4], 0xFF); // Padding
}

#[test]
fn test_define() {
    let source = ".define CON1 3\n.define CON2 0x0200\nLDI r0, CON1\nST (CON2), r0\n";
    let result = assemble(source, 0x0000, 0x3FFF).unwrap();

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
    let source = ".bank 0\n.org 0x0200\nFIXED_LABEL:\nNOP\n.bank 1\n.org 0x4100\nBANK_1_LABEL:\nNOP\nJMP FIXED_LABEL\n.bank 2\n.org 0x4200\nBANK_2_LABEL:\nNOP\n.org 0x6000\nJMP BANK_2_LABEL\nJMP FIXED_LABEL\n";
    let result = assemble(source, 0x0000, 0x7FFF).unwrap();

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
