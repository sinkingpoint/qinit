use nix::sys::stat::Mode;

pub fn to_mode(mode: String) -> Result<Mode, String> {
    if let Ok(bits) = u32::from_str_radix(&mode, 8) {
        if let Some(mode) = Mode::from_bits(bits) {
            return Ok(mode);
        }
    }

    return Err(format!("Invalid mode: {}", mode));
}
