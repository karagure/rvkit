const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const hal_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("hal/hal_test.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const run_hal_tests = b.addRunArtifact(hal_tests);
    const test_step = b.step("test", "Run HAL unit tests");
    test_step.dependOn(&run_hal_tests.step);
}
