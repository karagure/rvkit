pub struct Board {
    pub name: &'static str,
    /// `std.Target.Query` fields injected into the generated build.zig.
    pub target_query: &'static str,
    pub linker_script: &'static str,
    pub flash_tool: &'static str,
    /// Experimental boards scaffold and build, but `rvkit flash` refuses them.
    pub experimental: bool,
}

pub const CH32V003: Board = Board {
    name: "ch32v003",
    // QingKe V2A is RV32EC: 16 registers, compressed instructions, no hardware
    // multiply/divide. `e` and `i` are mutually exclusive base ISAs, so `i`
    // (implied by generic_rv32) must be removed.
    target_query: r"        .cpu_arch = .riscv32,
        .cpu_model = .{ .explicit = &std.Target.riscv.cpu.generic_rv32 },
        .cpu_features_add = std.Target.riscv.featureSet(&.{ .e, .c }),
        .cpu_features_sub = std.Target.riscv.featureSet(&.{ .i }),
        .os_tag = .freestanding,
        .abi = .none,",
    linker_script: include_str!("../../framework/linker/ch32v003.ld"),
    flash_tool: "wlink",
    experimental: false,
};

pub const ESP32_C3: Board = Board {
    name: "esp32-c3",
    // RV32IMC.
    target_query: r"        .cpu_arch = .riscv32,
        .cpu_model = .{ .explicit = &std.Target.riscv.cpu.generic_rv32 },
        .cpu_features_add = std.Target.riscv.featureSet(&.{ .m, .c }),
        .os_tag = .freestanding,
        .abi = .none,",
    linker_script: include_str!("../../framework/linker/esp32c3.ld"),
    flash_tool: "esptool",
    experimental: true,
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
