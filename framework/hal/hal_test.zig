//! Unit tests for the HAL comptime wrappers, using mock implementations.
//! Run with `zig build test` from the framework/ directory.

const std = @import("std");
const testing = std.testing;

const gpio = @import("gpio.zig");
const uart = @import("uart.zig");
const i2c = @import("i2c.zig");
const spi = @import("spi.zig");

test "hal entry point exposes all peripherals" {
    const hal = @import("hal.zig");
    comptime {
        _ = hal.gpio;
        _ = hal.i2c;
        _ = hal.spi;
        _ = hal.uart;
    }
}

const MockPin = struct {
    level: *gpio.Level,

    pub fn set_direction(self: MockPin, dir: gpio.Direction) void {
        _ = self;
        _ = dir;
    }

    pub fn set_pull(self: MockPin, pull: gpio.Pull) void {
        _ = self;
        _ = pull;
    }

    pub fn read(self: MockPin) gpio.Level {
        return self.level.*;
    }

    pub fn write(self: MockPin, level: gpio.Level) void {
        self.level.* = level;
    }

    pub fn toggle(self: MockPin) void {
        self.level.* = if (self.level.* == .high) .low else .high;
    }
};

test "gpio wrapper forwards to the implementation" {
    var level: gpio.Level = .low;
    const pin = gpio.Gpio(MockPin).init(.{ .level = &level });

    pin.set_direction(.output);
    pin.write(.high);
    try testing.expectEqual(gpio.Level.high, pin.read());
    pin.toggle();
    try testing.expectEqual(gpio.Level.low, pin.read());
}

const MockUart = struct {
    buf: []u8,
    len: *usize,

    pub fn configure(self: MockUart, config: uart.Config) void {
        _ = self;
        _ = config;
    }

    pub fn write_byte(self: MockUart, byte: u8) void {
        self.buf[self.len.*] = byte;
        self.len.* += 1;
    }

    pub fn read_byte(self: MockUart) ?u8 {
        _ = self;
        return null;
    }
};

test "uart write sends every byte in order" {
    var storage: [16]u8 = undefined;
    var len: usize = 0;
    const port = uart.Uart(MockUart).init(.{ .buf = &storage, .len = &len }, .{});

    port.write("hello");
    try testing.expectEqualStrings("hello", storage[0..len]);
    try testing.expectEqual(@as(?u8, null), port.read_byte());
}

const MockI2c = struct {
    last_addr: *u7,

    pub fn configure(self: MockI2c, config: i2c.Config) void {
        _ = self;
        _ = config;
    }

    pub fn write(self: MockI2c, addr: u7, data: []const u8) !void {
        _ = data;
        self.last_addr.* = addr;
    }

    pub fn read(self: MockI2c, addr: u7, buf: []u8) !void {
        self.last_addr.* = addr;
        for (buf) |*b| b.* = 0xAA;
    }

    pub fn write_read(self: MockI2c, addr: u7, write_data: []const u8, read_buf: []u8) !void {
        _ = write_data;
        self.last_addr.* = addr;
        for (read_buf) |*b| b.* = 0x55;
    }
};

test "i2c wrapper forwards address and buffers" {
    var last_addr: u7 = 0;
    const bus = i2c.I2c(MockI2c).init(.{ .last_addr = &last_addr }, .{});

    try bus.write(0x42, &.{ 1, 2, 3 });
    try testing.expectEqual(@as(u7, 0x42), last_addr);

    var buf: [2]u8 = undefined;
    try bus.read(0x17, &buf);
    try testing.expectEqual(@as(u7, 0x17), last_addr);
    try testing.expectEqualSlices(u8, &.{ 0xAA, 0xAA }, &buf);

    try bus.write_read(0x33, &.{9}, &buf);
    try testing.expectEqual(@as(u7, 0x33), last_addr);
    try testing.expectEqualSlices(u8, &.{ 0x55, 0x55 }, &buf);
}

const MockSpi = struct {
    pub fn configure(self: MockSpi, config: spi.Config) void {
        _ = self;
        _ = config;
    }

    pub fn transfer(self: MockSpi, tx: []const u8, rx: []u8) !void {
        _ = self;
        for (rx, 0..) |*b, idx| b.* = if (idx < tx.len) tx[idx] else 0;
    }

    pub fn write(self: MockSpi, data: []const u8) !void {
        _ = self;
        _ = data;
    }
};

test "spi transfer loops tx back into rx" {
    const bus = spi.Spi(MockSpi).init(.{}, .{});

    var rx: [3]u8 = undefined;
    try bus.transfer(&.{ 9, 8, 7 }, &rx);
    try testing.expectEqualSlices(u8, &.{ 9, 8, 7 }, &rx);
    try bus.write(&.{1});
}
