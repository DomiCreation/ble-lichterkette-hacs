//! Bluetooth protocol of the light sets. Same rules as
//! custom_components/ble_lichterkette/protocol.py; addresses in MECHANICS.md.

use uuid::Uuid;

/// Service the app scans for ("1000", -[ConnecTool startSearch] 0x10002f4bc).
pub const SERVICE: Uuid = Uuid::from_u128(0x00001000_0000_1000_8000_00805f9b34fb);
pub const ALT_SERVICE: Uuid = Uuid::from_u128(0x00002000_0000_1000_8000_00805f9b34fb);
/// Command characteristic "1001" (or "2001"), written without response.
pub const WRITE_CHARS: [Uuid; 2] = [
    Uuid::from_u128(0x00001001_0000_1000_8000_00805f9b34fb),
    Uuid::from_u128(0x00002001_0000_1000_8000_00805f9b34fb),
];

pub const CMD_ON: &str = "AD030100";
pub const CMD_OFF: &str = "AD030000";

/// The eight programs; the last one ("Steady") takes a brightness level.
pub const PROGRAMS: [(&str, &str); 7] = [
    ("Combination", "AD0101"),
    ("In waves", "AD0102"),
    ("Sequential", "AD0103"),
    ("Slow glow", "AD0104"),
    ("Chase/Flash", "AD0105"),
    // Device-tested: iPhone labels Twinkle/Flash=AD0106 and Slow fade=AD0107
    // are swapped on the light set.
    ("Twinkle/Flash", "AD0107"),
    ("Slow fade", "AD0106"),
];
pub const STEADY: &str = "Steady";

/// Brightness buttons: (label, level byte). The app stores level 1..4 for
/// 100/75/50/25 % and sends byte = 4 - level (jump table 0x1000365dc).
pub const BRIGHTNESS: [(&str, u8); 4] = [("100 %", 3), ("75 %", 2), ("50 %", 1), ("25 %", 0)];

const MARK: u8 = 0x54;

pub fn steady_command(level: u8) -> String {
    format!("AD0108{:02X}", level.min(3))
}

/// Hex text to bytes like -[BLEManager stringToByte:] 0x10001bc84:
/// uppercase, spaces removed, odd length or a non-hex digit gives nothing.
pub fn hex_to_bytes(text: &str) -> Option<Vec<u8>> {
    let clean: String = text.chars().filter(|c| *c != ' ').collect::<String>().to_uppercase();
    if clean.len() % 2 != 0 {
        return None;
    }
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).ok())
        .collect()
}

/// -[BLEManager encryption:] 0x10001b874:
/// bytes = hex("54" + command); n = len; frame = [0x54, n^0x54, k^0x54] + bytes[1..]^k
pub fn build_frame(command: &str, key: u8) -> Option<Vec<u8>> {
    let raw = hex_to_bytes(&format!("54{command}"))?;
    let mut out = vec![MARK, (raw.len() as u8) ^ MARK, key ^ MARK];
    out.extend(raw[1..].iter().map(|b| b ^ key));
    Some(out)
}

/// A key byte that changes every call (the app uses rand() * 255).
pub fn random_key() -> u8 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(0x9E37, Ordering::Relaxed);
    ((nanos ^ n ^ (nanos >> 8)) & 0xFF) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(v: &[u8]) -> String {
        v.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn frames_match_the_app() {
        assert_eq!(hex(&build_frame(CMD_ON, 0).unwrap()), "545154ad030100");
        assert_eq!(hex(&build_frame(CMD_ON, 0x5A).unwrap()), "54510ef7595b5a");
        assert_eq!(hex(&build_frame("AD0101", 0).unwrap()), "545054ad0101");
        assert!(build_frame("AD030", 0).is_none());
    }

    #[test]
    fn brightness_levels() {
        let cmds: Vec<String> = BRIGHTNESS.iter().map(|(_, l)| steady_command(*l)).collect();
        assert_eq!(cmds, ["AD010803", "AD010802", "AD010801", "AD010800"]);
    }

    #[test]
    fn twinkle_and_slow_fade_match_the_light_set() {
        assert_eq!(PROGRAMS[5], ("Twinkle/Flash", "AD0107"));
        assert_eq!(PROGRAMS[6], ("Slow fade", "AD0106"));
    }
}
