use casm::assemble;

const BANK_SIZE: usize = 16384;

#[test]
fn test_nop() {
    let source = "NOP\n";
    let result = assemble(source, 0x0000).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x00);
    assert_eq!(result[1], 0xFF); // Padding
}

#[test]
fn test_ldi() {
    let source = "LDI R1, 0x1234\n";
    let result = assemble(source, 0x0000).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x02); // LDI R1
    assert_eq!(result[1], 0x34); // low byte
    assert_eq!(result[2], 0x12); // high byte
    assert_eq!(result[3], 0xFF); // Padding
}

#[test]
fn test_jump() {
    let source = "START:\nNOP\nJMP START\n";
    let result = assemble(source, 0x0000).unwrap();

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
    let result = assemble(source, 0x0000).unwrap();

    assert_eq!(result.len(), BANK_SIZE * 2);
    assert_eq!(result[0], 0x01); // LDI r0
    assert_eq!(result[1], 0x03); // low byte of CON1
    assert_eq!(result[2], 0x00); // high byte of CON1
    assert_eq!(result[3], 0xF1); // ST (), r0
    assert_eq!(result[4], 0x00); // low byte of CON2
    assert_eq!(result[5], 0x02); // high byte of CON2
    assert_eq!(result[6], 0xFF); // Padding
}
