pub static EMULATOR_PATH: &str = "./emulator/koge29_h8-3069f_emulator";

pub fn check_version() -> Option<String> {
    let output = std::process::Command::new(EMULATOR_PATH)
        .arg("--version")
        .output();
    if let Ok(o) = output {
        if let Ok(os) = String::from_utf8(o.stdout) {
            if os.starts_with("koge29_h8-3069f_emulator ") {
                let version = os
                    .replace("koge29_h8-3069f_emulator ", "")
                    .trim()
                    .to_string();
                return Some(version);
            }
        }
    }
    return None;
}
