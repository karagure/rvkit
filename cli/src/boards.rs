pub struct Board {
    pub name: &'static str,
    pub cpu_arch: &'static str,
    pub linker_script: &'static str,
    pub flash_tool: &'static str,
}

pub const CH32V003: Board = Board {
    name: "ch32v003",
    cpu_arch: "riscv32",
    linker_script: include_str!("../../framework/linker/ch322v003.ld"),
    flash_tool: "wlink",
};

pub const ESP32_C3: Board = Board {
    name: "esp32-c3",
    cpu_arch: "riscv32",
    linker_script: include_str!("../../framework/linker/esp32c3.ld"),
    flash_tool: "esptool",
};

pub fn get(name: &str) -> Option<&'static Board> {
    match name {
        "ch32v003" => Some(&CH32V003),
        "esp32-c3" => Some(&ESP32_C3),
        _ => None,
    }
}

pub fn list() -> &'static [&'static Board] {
    &[&CH32V003, &ESP32_C3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_known_board_with_correct_metadata() {
        let b = get("ch32v003").expect("ch32v003 should be supported");
        assert_eq!(b.name, "ch32v003");
        assert_eq!(b.flash_tool, "wlink");
        assert_eq!(b.cpu_arch, "riscv32");
    }

    #[test]
    fn esp32c3_is_flashed_with_esptool() {
        assert_eq!(get("esp32-c3").unwrap().flash_tool, "esptool");
    }

    #[test]
    fn get_unknown_board_returns_none() {
        assert!(get("does-not-exist").is_none());
    }

    #[test]
    fn list_contains_every_supported_board() {
        let boards = list();
        assert_eq!(boards.len(), 2);
        assert!(boards.iter().any(|b| b.name == "ch32v003"));
        assert!(boards.iter().any(|b| b.name == "esp32-c3"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_known_board_with_correct_metadata() {
        let b = get("ch32v003").expect("ch32v003 should be supported");
        assert_eq!(b.name, "ch32v003");
        assert_eq!(b.flash_tool, "wlink");
        assert_eq!(b.cpu_arch, "riscv32");
    }

    #[test]
    fn esp32c3_is_flashed_with_esptool() {
        assert_eq!(get("esp32-c3").unwrap().flash_tool, "esptool");
    }

    #[test]
    fn get_unknown_board_returns_none() {
        assert!(get("does-not-exist").is_none());
    }

    #[test]
    fn list_contains_every_supported_board() {
        let boards = list();
        assert_eq!(boards.len(), 2);
        assert!(boards.iter().any(|b| b.name == "ch32v003"));
        assert!(boards.iter().any(|b| b.name == "esp32-c3"));
    }
}
