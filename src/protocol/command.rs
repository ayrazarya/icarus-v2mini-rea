//! Outbound 17-byte HID command construction and register writes.

use super::{
    CMD_APPLY_CONFIG, CMD_DONGLE_PAIR_CID_MID, CMD_ENTER_DONGLE_PAIR, CMD_ENTER_UPDATE_MODE,
    CMD_GET_4K_DONGLE_RGB, CMD_GET_DONGLE_RGB_BAR, CMD_GET_LONGRANGE_MODE, CMD_PC_DRIVER_STATUS,
    CMD_QUERY_BATTERY, CMD_QUERY_ONLINE, CMD_READ, CMD_READ_DONGLE_PAIR_STATUS,
    CMD_SET_4K_DONGLE_RGB, CMD_SET_DONGLE_RGB_BAR, CMD_SET_LONGRANGE_MODE, CMD_WRITE, PACKET_SIZE,
    REPORT_ID, REPORT_SIZE, addr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Command {
    /// Raw 17-byte payload
    pub data: [u8; REPORT_SIZE],
}

impl AsRef<[u8]> for Command {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Command {
    /// Build a read command for a specific register.
    #[must_use]
    pub const fn read(addr_hi: u8, addr_lo: u8, length: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_READ; // 0x08
        data[2] = 0x00; // flags
        data[3] = addr_hi;
        data[4] = addr_lo;
        data[5] = length;
        // Bytes 6-15 are zero (padding)
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Build a write command with payload data.
    #[must_use]
    pub const fn write(addr_hi: u8, addr_lo: u8, payload: &[u8]) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00; // flags
        data[3] = addr_hi;
        data[4] = addr_lo;
        let copy_len = if payload.len() < 10 {
            payload.len()
        } else {
            10
        };
        data[5] = copy_len as u8;

        let mut i = 0;
        while i < copy_len {
            data[6 + i] = payload[i];
            i += 1;
        }

        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Build a write command for a single 1-byte register.
    #[must_use]
    pub const fn write_byte(addr_hi: u8, addr_lo: u8, val: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00; // flags
        data[3] = addr_hi;
        data[4] = addr_lo;
        data[5] = 0x02; // length = 2 (val + payload checksum)
        data[6] = val;
        data[7] = Self::checksum(&[val]); // sub_1000aaf0: 0x55 - val
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Query battery status (opcode 0x04).
    #[must_use]
    pub const fn query_battery() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_QUERY_BATTERY; // 0x04
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Query online status (opcode 0x03).
    #[must_use]
    pub const fn query_online() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_QUERY_ONLINE; // 0x03
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Build a PC Driver Status command (opcode 0x02) to notify MCU the driver is connected.
    #[must_use]
    pub const fn pc_driver_status(active: bool) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_PC_DRIVER_STATUS; // 0x02
        data[5] = 0x01; // length = 1
        data[6] = if active { 0x01 } else { 0x00 };
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Compute 8-bit checksum: 0x55 - wrapping_sum(bytes).
    /// Decompiled from 0x1000aaf0 in HIDUsb.dll / Compx mouse firmware.
    #[must_use]
    pub const fn checksum(bytes: &[u8]) -> u8 {
        let mut sum = 0u8;
        let mut i = 0;
        while i < bytes.len() {
            sum = sum.wrapping_add(bytes[i]);
            i += 1;
        }
        0x55u8.wrapping_sub(sum)
    }

    /// Get the full 17-byte packet ready for HID transmission.
    #[must_use]
    pub const fn to_packet(&self) -> [u8; PACKET_SIZE] {
        self.data
    }
}

impl Command {
    /// Read current polling rate.
    #[must_use]
    pub const fn read_report_rate() -> Self {
        Self::read(addr::READ_REPORT_RATE.0, addr::READ_REPORT_RATE.1, 0x02)
    }

    /// Read max DPI stages count.
    #[must_use]
    pub const fn read_max_dpi_stages() -> Self {
        Self::read(
            addr::READ_MAX_DPI_STAGES.0,
            addr::READ_MAX_DPI_STAGES.1,
            0x02,
        )
    }

    /// Read current DPI stage index.
    #[must_use]
    pub const fn read_current_dpi() -> Self {
        Self::read(addr::READ_CURRENT_DPI.0, addr::READ_CURRENT_DPI.1, 0x02)
    }

    /// Read DPI LED settings.
    #[must_use]
    pub const fn read_dpi_led() -> Self {
        Self::read(addr::DPI_LED.0, addr::DPI_LED.1, 0x07)
    }

    /// Read LED bar colors.
    #[must_use]
    pub const fn read_led_bar() -> Self {
        Self::read(0x00, 0xA0, 0x07)
    }

    /// Legacy read battery level.
    #[must_use]
    pub const fn read_battery() -> Self {
        Self::read(addr::BATTERY.0, addr::BATTERY.1, 0x01)
    }

    /// Read firmware version.
    #[must_use]
    pub const fn read_version() -> Self {
        Self::read(addr::VERSION.0, addr::VERSION.1, 0x08)
    }

    /// Write polling rate.
    #[must_use]
    pub const fn write_report_rate(code: u8) -> Self {
        Self::write_byte(addr::REPORT_RATE.0, addr::REPORT_RATE.1, code)
    }

    /// Write max DPI stages.
    #[must_use]
    pub const fn write_max_dpi_stages(count: u8) -> Self {
        Self::write_byte(addr::MAX_DPI_STAGES.0, addr::MAX_DPI_STAGES.1, count)
    }

    /// Write current DPI stage.
    #[must_use]
    pub const fn write_current_dpi(stage: u8) -> Self {
        Self::write_byte(addr::CURRENT_DPI.0, addr::CURRENT_DPI.1, stage)
    }

    /// Write composite DPI LED settings to hardware register (0x00, 0xA0).
    #[must_use]
    pub const fn write_dpi_led_composite(
        mode: u8,
        brightness: u8,
        breath_speed: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00;
        data[3] = addr::DPI_LED_COMPOSITE.0; // 0x00
        data[4] = addr::DPI_LED_COMPOSITE.1; // 0xA0
        data[5] = 0x07; // length = 7
        data[6] = mode;
        data[7] = brightness;
        data[8] = breath_speed;
        data[9] = r;
        data[10] = g;
        data[11] = b;
        data[12] = Self::checksum(&[mode, brightness, breath_speed, r, g, b]);
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Write DPI LED mode register (0x00, 0x4C).
    #[must_use]
    pub const fn write_dpi_led_mode(mode: u8) -> Self {
        Self::write_byte(addr::DPI_LED_MODE.0, addr::DPI_LED_MODE.1, mode)
    }

    /// Write DPI LED brightness register (0x00, 0x4E).
    #[must_use]
    pub const fn write_dpi_led_brightness(brightness: u8) -> Self {
        Self::write_byte(
            addr::DPI_LED_BRIGHTNESS.0,
            addr::DPI_LED_BRIGHTNESS.1,
            brightness,
        )
    }

    /// Write DPI LED breath speed register (0x00, 0x50).
    #[must_use]
    pub const fn write_dpi_led_speed(speed: u8) -> Self {
        Self::write_byte(addr::DPI_LED_SPEED.0, addr::DPI_LED_SPEED.1, speed)
    }

    /// Write DPI LED param4 register (0x00, 0x52).
    #[must_use]
    pub const fn write_dpi_led_param4(val: u8) -> Self {
        Self::write_byte(addr::DPI_LED_PARAM4.0, addr::DPI_LED_PARAM4.1, val)
    }

    /// Write full DPI LED settings matching HIDUsb.dll / Compx protocol.
    /// Writes direct registers (0x4C, 0x4E, 0x50, 0x52) and composite register (0xA0).
    #[must_use]
    pub const fn write_dpi_led_commands(
        mode: u8,
        brightness: u8,
        breath_speed: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> [Self; 5] {
        let scaled = (brightness as u16 * 255) / 10;
        let bright_byte = if scaled > 255 { 255 } else { scaled as u8 };
        [
            Self::write_dpi_led_mode(mode),
            Self::write_dpi_led_brightness(bright_byte),
            Self::write_dpi_led_speed(breath_speed),
            Self::write_dpi_led_param4(0x01),
            Self::write_dpi_led_composite(mode, brightness, breath_speed, r, g, b),
        ]
    }

    /// Single write_dpi_led packet using composite hardware register (0x00, 0xA0).
    #[must_use]
    pub const fn write_dpi_led(
        mode: u8,
        brightness: u8,
        breath_speed: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Self {
        Self::write_dpi_led_composite(mode, brightness, breath_speed, r, g, b)
    }

    /// Read lift-off distance (LOD).
    #[must_use]
    pub const fn read_lod() -> Self {
        Self::read(addr::READ_LOD.0, addr::READ_LOD.1, 0x02)
    }

    /// Write lift-off distance (LOD).
    #[must_use]
    pub const fn write_lod(code: u8) -> Self {
        Self::write_byte(addr::LOD.0, addr::LOD.1, code)
    }

    /// Read debounce time.
    #[must_use]
    pub const fn read_debounce() -> Self {
        Self::read(addr::READ_DEBOUNCE.0, addr::READ_DEBOUNCE.1, 0x02)
    }

    /// Write debounce time filter.
    #[must_use]
    pub const fn write_debounce(ms: u8) -> Self {
        Self::write_byte(addr::DEBOUNCE.0, addr::DEBOUNCE.1, ms)
    }

    /// Read angle snapping.
    #[must_use]
    pub const fn read_angle_snap() -> Self {
        Self::read(addr::READ_ANGLE_SNAP.0, addr::READ_ANGLE_SNAP.1, 0x02)
    }

    /// Write angle snapping.
    #[must_use]
    pub const fn write_angle_snap(enabled: bool) -> Self {
        Self::write_byte(
            addr::ANGLE_SNAP.0,
            addr::ANGLE_SNAP.1,
            if enabled { 0x01 } else { 0x00 },
        )
    }

    /// Read motion sync.
    #[must_use]
    pub const fn read_motion_sync() -> Self {
        Self::read(addr::READ_MOTION_SYNC.0, addr::READ_MOTION_SYNC.1, 0x02)
    }

    /// Write motion sync.
    #[must_use]
    pub const fn write_motion_sync(enabled: bool) -> Self {
        Self::write_byte(
            addr::MOTION_SYNC.0,
            addr::MOTION_SYNC.1,
            if enabled { 0x01 } else { 0x00 },
        )
    }

    /// Read ripple control.
    #[must_use]
    pub const fn read_ripple() -> Self {
        Self::read(addr::READ_RIPPLE.0, addr::READ_RIPPLE.1, 0x02)
    }

    /// Write ripple control.
    #[must_use]
    pub const fn write_ripple(enabled: bool) -> Self {
        Self::write_byte(
            addr::RIPPLE.0,
            addr::RIPPLE.1,
            if enabled { 0x01 } else { 0x00 },
        )
    }

    /// Read sensor mode (performance vs office).
    #[must_use]
    pub const fn read_sensor_mode() -> Self {
        Self::read(addr::READ_SENSOR_MODE.0, addr::READ_SENSOR_MODE.1, 0x02)
    }

    /// Write sensor mode (0x00 = Performance, 0x01 = Office).
    #[must_use]
    pub const fn write_sensor_mode(performance: bool) -> Self {
        Self::write_byte(
            addr::SENSOR_MODE.0,
            addr::SENSOR_MODE.1,
            if performance { 0x00 } else { 0x01 },
        )
    }

    /// Read sensor sleep timer.
    #[must_use]
    pub const fn read_sensor_sleep() -> Self {
        Self::read(addr::READ_SENSOR_SLEEP.0, addr::READ_SENSOR_SLEEP.1, 0x02)
    }

    /// Write sensor sleep timer.
    #[must_use]
    pub const fn write_sensor_sleep(seconds: u32) -> Self {
        let code = if seconds == 0 {
            0x00
        } else {
            let tens = seconds / 10;
            if tens < 1 {
                1
            } else if tens > 60 {
                60
            } else {
                tens as u8
            }
        };
        Self::write_byte(addr::SENSOR_SLEEP.0, addr::SENSOR_SLEEP.1, code)
    }

    /// Write 3-byte button mapping for a specific button index (0..5).
    #[must_use]
    pub const fn write_button(idx: u8, action_bytes: [u8; 3]) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_WRITE;
        data[2] = 0x00;
        data[3] = 0x00; // addr_hi = 0x00
        data[4] = addr::BUTTON_BASE + idx * 4;
        data[5] = 0x04;
        data[6] = action_bytes[0];
        data[7] = action_bytes[1];
        data[8] = action_bytes[2];
        data[9] = Self::checksum(&action_bytes);
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Write DPI stage value to hardware register (0x00, 0x0C + stage * 4).
    #[must_use]
    pub const fn write_dpi_stage(stage: u8, x_reg: u8, y_reg: u8) -> Self {
        Self::write_dpi_stage_ext(stage, x_reg, y_reg, 0x00)
    }

    /// Write DPI stage value with 10-bit high flags to hardware register (0x00, 0x0C + stage * 4).
    #[must_use]
    pub const fn write_dpi_stage_ext(stage: u8, x_reg: u8, y_reg: u8, flag: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00;
        data[3] = 0x00; // addr_hi = 0x00
        data[4] = addr::DPI_STAGE_BASE + stage * 4;
        data[5] = 0x04; // length = 4
        data[6] = x_reg;
        data[7] = y_reg;
        data[8] = flag;
        data[9] = Self::checksum(&[x_reg, y_reg, flag]);
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Write DPI stage LED color to hardware register (0x00, 0x2C + stage * 4).
    #[must_use]
    pub const fn write_dpi_stage_color(stage: u8, r: u8, g: u8, b: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_WRITE;
        data[2] = 0x00;
        data[3] = 0x00; // addr_hi = 0x00
        data[4] = addr::DPI_COLOR_BASE + stage * 4;
        data[5] = 0x04;
        data[6] = r;
        data[7] = g;
        data[8] = b;
        data[9] = Self::checksum(&[r, g, b]);
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Apply and commit configuration profile (opcode 0x0F, profile index e.g. 1).
    #[must_use]
    pub const fn apply_config(profile: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_APPLY_CONFIG;
        data[5] = 0x01;
        data[6] = profile;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    // ── Extended Commands (2.4G Pairing, Long Range Mode, 4K Dongle, Update) ───

    /// Enter 2.4G dongle pairing mode (opcode 0x06).
    #[must_use]
    pub const fn enter_dongle_pair() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_ENTER_DONGLE_PAIR;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Enter 2.4G dongle pairing with specific CID and MID (opcode 0x05).
    #[must_use]
    pub const fn enter_dongle_pair_with_cid_mid(cid: u8, mid: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_DONGLE_PAIR_CID_MID;
        data[5] = 0x02;
        data[6] = cid;
        data[7] = mid;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Read 2.4G dongle pairing status (opcode 0x09).
    #[must_use]
    pub const fn read_dongle_pair_status() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_READ_DONGLE_PAIR_STATUS;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Set Long Range wireless mode (opcode 0x16, RF transmission power boost).
    #[must_use]
    pub const fn set_long_range_mode(enabled: bool) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_SET_LONGRANGE_MODE;
        data[5] = 0x01;
        data[6] = if enabled { 0x01 } else { 0x00 };
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Read Long Range wireless mode status (opcode 0x17).
    #[must_use]
    pub const fn read_long_range_mode() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_GET_LONGRANGE_MODE;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Enter USB firmware upgrade mode (opcode 0x0E, ISP/IAP).
    #[must_use]
    pub const fn enter_update_mode() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_ENTER_UPDATE_MODE;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Set 4K Dongle RGB parameters (opcode 0x14).
    #[must_use]
    pub const fn set_4k_dongle_rgb(payload: &[u8]) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_SET_4K_DONGLE_RGB;
        let copy_len = if payload.len() < 10 {
            payload.len()
        } else {
            10
        };
        data[5] = copy_len as u8;
        let mut i = 0;
        while i < copy_len {
            data[6 + i] = payload[i];
            i += 1;
        }
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Query 4K Dongle RGB parameters (opcode 0x15).
    #[must_use]
    pub const fn read_4k_dongle_rgb() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_GET_4K_DONGLE_RGB;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Set Dongle RGB Bar mode (opcode 0x18).
    #[must_use]
    pub const fn set_dongle_rgb_bar_mode(payload: &[u8]) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_SET_DONGLE_RGB_BAR;
        let copy_len = if payload.len() < 10 {
            payload.len()
        } else {
            10
        };
        data[5] = copy_len as u8;
        let mut i = 0;
        while i < copy_len {
            data[6 + i] = payload[i];
            i += 1;
        }
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }

    /// Query Dongle RGB Bar mode (opcode 0x19).
    #[must_use]
    pub const fn read_dongle_rgb_bar_mode() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_GET_DONGLE_RGB_BAR;
        let (head, _) = data.split_at(16);
        data[16] = Self::checksum(head);
        Self { data }
    }
}
