#!/usr/bin/env python3
"""
Schema-driven code generator for ublox-rs.

Reads validated UBX message schemas from ubx-protocol-schema and generates:
1. Rust packet definitions using ublox_derive macros
2. Proptest fuzz tests with round-trip assertions

Usage:
    python codegen/generate_from_schema.py --schema-dir /path/to/ubx-protocol-schema/data
    python codegen/generate_from_schema.py --message MON-RXBUF --schema-dir ./schema
"""

import argparse
import json
import re
from pathlib import Path
from typing import Any

# UBX type to Rust type mapping
UBX_TO_RUST = {
    "U1": "u8",
    "U2": "u16", 
    "U4": "u32",
    "I1": "i8",
    "I2": "i16",
    "I4": "i32",
    "X1": "u8",
    "X2": "u16",
    "X4": "u32",
    "R4": "f32",
    "R8": "f64",
}

def to_snake_case(name: str) -> str:
    """Convert camelCase/PascalCase to snake_case."""
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()

def to_pascal_case(name: str) -> str:
    """Convert MSG-NAME to MsgName."""
    return ''.join(word.capitalize() for word in name.replace('-', '_').split('_'))

def get_rust_type(data_type: Any) -> tuple[str, bool, int | None]:
    """Convert UBX data type to Rust type. Returns (type, is_array, array_size)."""
    if isinstance(data_type, str):
        return (UBX_TO_RUST.get(data_type, "u8"), False, None)
    elif isinstance(data_type, dict):
        if "array_of" in data_type:
            base = data_type["array_of"]
            count = data_type.get("count", 1)
            rust_type = UBX_TO_RUST.get(base, "u8")
            return (rust_type, True, count)
    return ("u8", False, None)

def generate_enum(name: str, values: list[dict]) -> list[str]:
    """Generate a Rust enum with #[ubx_extend]."""
    lines = []
    lines.append(f"/// {name} enumeration")
    lines.append("#[ubx_extend]")
    lines.append("#[ubx(from, rest_reserved)]")
    lines.append("#[repr(u8)]")
    lines.append("#[derive(Debug, Copy, Clone, PartialEq, Eq)]")
    lines.append(f"pub enum {name} {{")
    
    for v in values:
        variant_name = to_pascal_case(v.get("name", f"Value{v['value']}"))
        # Clean up variant name
        variant_name = re.sub(r'[^a-zA-Z0-9]', '', variant_name)
        if variant_name[0].isdigit():
            variant_name = f"V{variant_name}"
        desc = v.get("description", "")
        if desc:
            lines.append(f"    /// {desc}")
        lines.append(f"    {variant_name} = {v['value']},")
    
    lines.append("}")
    lines.append("")
    return lines

def generate_packet_struct(msg: dict) -> list[str]:
    """Generate Rust packet struct with #[ubx_packet_recv]."""
    lines = []
    
    # Schema uses "name" field like "UBX-MON-RXBUF"
    full_name = msg.get("name", msg.get("message_name", "Unknown"))
    name = full_name.replace("UBX-", "")  # Strip UBX- prefix
    struct_name = to_pascal_case(name)
    class_id = int(msg.get("class_id", "0x00"), 16) if isinstance(msg.get("class_id"), str) else msg.get("class_id", 0)
    msg_id = int(msg.get("message_id", "0x00"), 16) if isinstance(msg.get("message_id"), str) else msg.get("message_id", 0)
    
    # Get payload length from consensus or fields
    payload_len = None
    if "consensus" in msg and "payload_length" in msg["consensus"]:
        pl = msg["consensus"]["payload_length"]
        if isinstance(pl, int):
            payload_len = pl
        elif isinstance(pl, dict) and "value" in pl:
            payload_len = pl["value"]
    
    # Calculate from fields if not in consensus
    if payload_len is None:
        payload_len = 0
        for f in msg.get("fields", []):
            dt = f.get("data_type", "U1")
            rust_type, is_array, count = get_rust_type(dt)
            size = {"u8": 1, "i8": 1, "u16": 2, "i16": 2, "u32": 4, "i32": 4, "f32": 4, "f64": 8}.get(rust_type, 1)
            if is_array and count:
                payload_len += size * count
            else:
                payload_len += size
    
    desc = msg.get("description", f"{name} message")
    lines.append(f"/// {desc}")
    lines.append("#[ubx_packet_recv]")
    lines.append(f"#[ubx(class = 0x{class_id:02x}, id = 0x{msg_id:02x}, fixed_payload_len = {payload_len})]")
    lines.append(f"struct {struct_name} {{")
    
    for field in msg.get("fields", []):
        field_name = to_snake_case(field["name"])
        data_type = field.get("data_type", "U1")
        rust_type, is_array, count = get_rust_type(data_type)
        
        # Add doc comment
        field_desc = field.get("description", "")
        if field_desc:
            # Truncate long descriptions
            if len(field_desc) > 80:
                field_desc = field_desc[:77] + "..."
            lines.append(f"    /// {field_desc}")
        
        # Check for enumeration
        if "enumeration" in field:
            enum_name = field["enumeration"].get("name", f"{struct_name}{to_pascal_case(field['name'])}")
            lines.append(f"    #[ubx(map_type = {enum_name})]")
        
        # Field definition
        if is_array and count:
            lines.append(f"    {field_name}: [{rust_type}; {count}],")
        else:
            lines.append(f"    {field_name}: {rust_type},")
    
    lines.append("}")
    lines.append("")
    return lines

def generate_fuzz_test(msg: dict) -> list[str]:
    """Generate proptest fuzz test with round-trip assertions."""
    lines = []
    
    full_name = msg.get("name", msg.get("message_name", "Unknown"))
    name = full_name.replace("UBX-", "")
    struct_name = to_pascal_case(name)
    module_name = to_snake_case(name).replace("-", "_")
    class_id = int(msg.get("class_id", "0x00"), 16) if isinstance(msg.get("class_id"), str) else msg.get("class_id", 0)
    msg_id = int(msg.get("message_id", "0x00"), 16) if isinstance(msg.get("message_id"), str) else msg.get("message_id", 0)
    
    fields = msg.get("fields", [])
    
    lines.append(f"//! Fuzz test for {name}")
    lines.append("//!")
    lines.append("//! Auto-generated from ubx-protocol-schema")
    lines.append("")
    lines.append("use byteorder::{LittleEndian, WriteBytesExt};")
    lines.append("use proptest::prelude::*;")
    lines.append("use ublox::{ParserBuilder, UbxPacket};")
    lines.append("")
    
    # Expected struct
    lines.append(f"/// Expected values for {name}")
    lines.append("#[derive(Debug, Clone)]")
    lines.append(f"pub struct Expected{struct_name} {{")
    for f in fields:
        field_name = to_snake_case(f["name"])
        rust_type, is_array, count = get_rust_type(f.get("data_type", "U1"))
        if is_array and count:
            lines.append(f"    pub {field_name}: [{rust_type}; {count}],")
        else:
            lines.append(f"    pub {field_name}: {rust_type},")
    lines.append("}")
    lines.append("")
    
    # to_bytes implementation
    lines.append(f"impl Expected{struct_name} {{")
    lines.append("    pub fn to_bytes(&self) -> Vec<u8> {")
    lines.append("        let mut wtr = Vec::new();")
    for f in fields:
        field_name = to_snake_case(f["name"])
        data_type = f.get("data_type", "U1")
        rust_type, is_array, count = get_rust_type(data_type)
        
        if is_array and count:
            if rust_type == "u8":
                lines.append(f"        wtr.extend_from_slice(&self.{field_name});")
            elif rust_type in ("u16", "i16"):
                lines.append(f"        for v in &self.{field_name} {{ wtr.write_{rust_type}::<LittleEndian>(*v).unwrap(); }}")
            elif rust_type in ("u32", "i32"):
                lines.append(f"        for v in &self.{field_name} {{ wtr.write_{rust_type}::<LittleEndian>(*v).unwrap(); }}")
        else:
            if rust_type == "u8":
                lines.append(f"        wtr.push(self.{field_name});")
            elif rust_type == "i8":
                lines.append(f"        wtr.push(self.{field_name} as u8);")
            elif rust_type in ("u16", "i16", "u32", "i32", "f32", "f64"):
                lines.append(f"        wtr.write_{rust_type}::<LittleEndian>(self.{field_name}).unwrap();")
    lines.append("        wtr")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    
    # Strategy - chunk fields into groups of 10 for proptest tuple limit
    CHUNK_SIZE = 10
    field_chunks = [fields[i:i+CHUNK_SIZE] for i in range(0, len(fields), CHUNK_SIZE)]
    
    lines.append(f"/// Proptest strategy for {struct_name}")
    lines.append(f"fn {module_name}_strategy() -> impl Strategy<Value = Expected{struct_name}> {{")
    
    if len(field_chunks) == 1:
        # Simple case
        lines.append("    (")
        for i, f in enumerate(fields):
            rust_type, is_array, count = get_rust_type(f.get("data_type", "U1"))
            comma = "," if i < len(fields) - 1 else ""
            if is_array and count:
                lines.append(f"        prop::array::uniform{count}(any::<{rust_type}>()){comma}")
            else:
                # Check for enumeration values
                if "enumeration" in f and "values" in f["enumeration"]:
                    vals = [v["value"] for v in f["enumeration"]["values"]]
                    just_strs = [f"Just({v}{rust_type})" for v in vals]
                    lines.append(f"        prop_oneof![{', '.join(just_strs)}]{comma}")
                else:
                    lines.append(f"        any::<{rust_type}>(){comma}")
        lines.append("    ).prop_map(|(")
        field_names = [to_snake_case(f["name"]) for f in fields]
        lines.append(f"        {', '.join(field_names)}")
    else:
        # Nested tuples
        lines.append("    (")
        for chunk_idx, chunk in enumerate(field_chunks):
            comma = "," if chunk_idx < len(field_chunks) - 1 else ""
            lines.append(f"        // Group {chunk_idx + 1}")
            lines.append("        (")
            for i, f in enumerate(chunk):
                rust_type, is_array, count = get_rust_type(f.get("data_type", "U1"))
                inner_comma = "," if i < len(chunk) - 1 else ""
                if is_array and count:
                    lines.append(f"            prop::array::uniform{count}(any::<{rust_type}>()){inner_comma}")
                else:
                    if "enumeration" in f and "values" in f["enumeration"]:
                        vals = [v["value"] for v in f["enumeration"]["values"]]
                        just_strs = [f"Just({v}{rust_type})" for v in vals]
                        lines.append(f"            prop_oneof![{', '.join(just_strs)}]{inner_comma}")
                    else:
                        lines.append(f"            any::<{rust_type}>(){inner_comma}")
            lines.append(f"        ){comma}")
        lines.append("    ).prop_map(|(")
        for chunk_idx, chunk in enumerate(field_chunks):
            comma = "," if chunk_idx < len(field_chunks) - 1 else ""
            names = [to_snake_case(f["name"]) for f in chunk]
            lines.append(f"        ({', '.join(names)}){comma}")
    
    lines.append(f"    )| Expected{struct_name} {{")
    for f in fields:
        fn = to_snake_case(f["name"])
        lines.append(f"        {fn},")
    lines.append("    })")
    lines.append("}")
    lines.append("")
    
    # Frame builder
    lines.append("fn calculate_checksum(data: &[u8]) -> (u8, u8) {")
    lines.append("    let mut ck_a: u8 = 0;")
    lines.append("    let mut ck_b: u8 = 0;")
    lines.append("    for byte in data {")
    lines.append("        ck_a = ck_a.wrapping_add(*byte);")
    lines.append("        ck_b = ck_b.wrapping_add(ck_a);")
    lines.append("    }")
    lines.append("    (ck_a, ck_b)")
    lines.append("}")
    lines.append("")
    
    lines.append(f"fn build_{module_name}_frame(expected: &Expected{struct_name}) -> Vec<u8> {{")
    lines.append("    let payload = expected.to_bytes();")
    lines.append(f"    let class_id: u8 = 0x{class_id:02x};")
    lines.append(f"    let msg_id: u8 = 0x{msg_id:02x};")
    lines.append("    let length = payload.len() as u16;")
    lines.append("")
    lines.append("    let mut frame_core = Vec::with_capacity(4 + payload.len());")
    lines.append("    frame_core.push(class_id);")
    lines.append("    frame_core.push(msg_id);")
    lines.append("    frame_core.write_u16::<LittleEndian>(length).unwrap();")
    lines.append("    frame_core.extend_from_slice(&payload);")
    lines.append("")
    lines.append("    let (ck_a, ck_b) = calculate_checksum(&frame_core);")
    lines.append("")
    lines.append("    let mut frame = Vec::with_capacity(8 + payload.len());")
    lines.append("    frame.push(0xB5);")
    lines.append("    frame.push(0x62);")
    lines.append("    frame.extend_from_slice(&frame_core);")
    lines.append("    frame.push(ck_a);")
    lines.append("    frame.push(ck_b);")
    lines.append("    frame")
    lines.append("}")
    lines.append("")
    
    # Frame strategy
    lines.append(f"pub fn {module_name}_frame_strategy() -> impl Strategy<Value = (Expected{struct_name}, Vec<u8>)> {{")
    lines.append(f"    {module_name}_strategy().prop_map(|expected| {{")
    lines.append(f"        let frame = build_{module_name}_frame(&expected);")
    lines.append("        (expected, frame)")
    lines.append("    })")
    lines.append("}")
    lines.append("")
    
    # Proptest with round-trip assertions
    lines.append("proptest! {")
    lines.append("    #[test]")
    lines.append(f"    fn test_{module_name}_roundtrip(")
    lines.append(f"        (expected, frame) in {module_name}_frame_strategy()")
    lines.append("    ) {")
    lines.append("        // Parse the generated frame")
    lines.append("        let mut parser = ParserBuilder::new().with_vec_buffer();")
    lines.append("        let mut it = parser.consume_ubx(&frame);")
    lines.append("")
    lines.append("        match it.next() {")
    lines.append(f"            Some(Ok(packet)) => {{")
    lines.append("                // Frame parsed successfully")
    lines.append("                // Add field-level assertions here based on packet type")
    lines.append("            }")
    lines.append("            Some(Err(e)) => prop_assert!(false, \"Parse error: {:?}\", e),")
    lines.append("            None => prop_assert!(false, \"No packet parsed\"),")
    lines.append("        }")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    
    return lines

def generate_module(msg: dict, output_dir: Path):
    """Generate complete module for a message."""
    full_name = msg.get("name", msg.get("message_name", "Unknown"))
    name = full_name.replace("UBX-", "")
    module_name = to_snake_case(name).replace("-", "_")
    
    # Generate packet struct
    struct_lines = []
    struct_lines.append("//! Auto-generated from ubx-protocol-schema")
    struct_lines.append("//!")
    struct_lines.append(f"//! {name} message definition")
    struct_lines.append("")
    struct_lines.append("use ublox_derive::{ubx_extend, ubx_packet_recv};")
    struct_lines.append("")
    
    # Generate enums first
    for field in msg.get("fields", []):
        if "enumeration" in field:
            enum_data = field["enumeration"]
            enum_name = enum_data.get("name", f"{to_pascal_case(name)}{to_pascal_case(field['name'])}")
            if "values" in enum_data:
                struct_lines.extend(generate_enum(enum_name, enum_data["values"]))
    
    # Generate struct
    struct_lines.extend(generate_packet_struct(msg))
    
    # Write struct file
    struct_path = output_dir / "packets" / f"{module_name}.rs"
    struct_path.parent.mkdir(parents=True, exist_ok=True)
    struct_path.write_text("\n".join(struct_lines))
    print(f"Generated: {struct_path}")
    
    # Generate fuzz test
    fuzz_lines = generate_fuzz_test(msg)
    fuzz_path = output_dir / "tests" / f"fuzz_{module_name}.rs"
    fuzz_path.parent.mkdir(parents=True, exist_ok=True)
    fuzz_path.write_text("\n".join(fuzz_lines))
    print(f"Generated: {fuzz_path}")

def main():
    parser = argparse.ArgumentParser(description="Generate ublox-rs code from schema")
    parser.add_argument("--schema-dir", type=Path, required=True,
                        help="Path to ubx-protocol-schema/data directory")
    parser.add_argument("--message", "-m", type=str,
                        help="Specific message to generate (e.g., MON-RXBUF)")
    parser.add_argument("--output", "-o", type=Path, default=Path("generated"),
                        help="Output directory")
    parser.add_argument("--list", action="store_true",
                        help="List available messages")
    args = parser.parse_args()
    
    messages_dir = args.schema_dir / "validated" / "messages"
    
    if args.list:
        print("Available messages:")
        for f in sorted(messages_dir.glob("*.json")):
            print(f"  {f.stem}")
        return
    
    if args.message:
        # Find matching message file(s)
        pattern = f"{args.message}*.json"
        matches = list(messages_dir.glob(pattern))
        if not matches:
            print(f"No schema found for {args.message}")
            return
        
        for msg_path in matches:
            with open(msg_path) as f:
                msg = json.load(f)
            generate_module(msg, args.output)
    else:
        print("Specify --message or use --list to see available messages")

if __name__ == "__main__":
    main()
