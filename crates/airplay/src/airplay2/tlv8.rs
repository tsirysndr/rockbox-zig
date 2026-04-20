/// TLV8 type codes used in HAP pairing (AirPlay 2 / HomeKit).
pub mod types {
    pub const METHOD: u8 = 0x00;
    pub const IDENTIFIER: u8 = 0x01;
    pub const SALT: u8 = 0x02;
    pub const PUBLIC_KEY: u8 = 0x03;
    pub const PROOF: u8 = 0x04;
    pub const ENCRYPTED_DATA: u8 = 0x05;
    pub const STATE: u8 = 0x06;
    pub const ERROR: u8 = 0x07;
    pub const SIGNATURE: u8 = 0x0A;
    pub const SEPARATOR: u8 = 0xFF;
}

pub struct Tlv8Item {
    pub typ: u8,
    pub value: Vec<u8>,
}

/// Encode a slice of TLV8 items. Values > 255 bytes are split across fragments
/// with the same type byte (as required by the HAP spec).
pub fn encode(items: &[Tlv8Item]) -> Vec<u8> {
    let mut out = Vec::new();
    for item in items {
        if item.value.is_empty() {
            out.push(item.typ);
            out.push(0u8);
        } else {
            for chunk in item.value.chunks(255) {
                out.push(item.typ);
                out.push(chunk.len() as u8);
                out.extend_from_slice(chunk);
            }
        }
    }
    out
}

/// Decode a TLV8 byte stream. Adjacent fragments with the same type are merged.
pub fn decode(data: &[u8]) -> Vec<Tlv8Item> {
    let mut items: Vec<Tlv8Item> = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() {
        let typ = data[i];
        let len = data[i + 1] as usize;
        i += 2;
        let end = (i + len).min(data.len());
        let value = data[i..end].to_vec();
        i = end;
        if let Some(last) = items.last_mut() {
            if last.typ == typ {
                last.value.extend_from_slice(&value);
                continue;
            }
        }
        items.push(Tlv8Item { typ, value });
    }
    items
}

pub fn find(items: &[Tlv8Item], typ: u8) -> Option<&[u8]> {
    items.iter().find(|i| i.typ == typ).map(|i| i.value.as_slice())
}
