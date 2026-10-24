//! Flash Data Map Registers (Decompiled from HIDUsb.dll & functionset.obj).

// addr_hi is always 0x00 for all flash map registers
pub const REPORT_RATE: (u8, u8) = (0x00, 0x00);

pub const MAX_DPI_STAGES: (u8, u8) = (0x00, 0x02);
pub const CURRENT_DPI: (u8, u8) = (0x00, 0x04);
pub const ANGLE_SNAP_X: (u8, u8) = (0x00, 0x06);
pub const ANGLE_SNAP_Y: (u8, u8) = (0x00, 0x08);
pub const LOD: (u8, u8) = (0x00, 0x0A);
pub const DPI_STAGE_BASE: u8 = 0x0C; // 0x0C + stage * 4, length 4: [x_reg, y_reg, 0x00, chk]
pub const DPI_COLOR_BASE: u8 = 0x2C; // 0x2C + stage * 4, length 4: [r, g, b, chk]
pub const DPI_LED_MODE: (u8, u8) = (0x00, 0x4C); // length 2: [mode, chk]
pub const DPI_LED_BRIGHTNESS: (u8, u8) = (0x00, 0x4E); // length 2: [bright, chk]
pub const DPI_LED_SPEED: (u8, u8) = (0x00, 0x50); // length 2: [speed, chk]
pub const DPI_LED_PARAM4: (u8, u8) = (0x00, 0x52); // length 2: [p4, chk]
pub const BUTTON_BASE: u8 = 0x60; // 0x60 + idx * 4, length 4: [b0, b1, b2, chk]
pub const DPI_LED_COMPOSITE: (u8, u8) = (0x00, 0xA0); // length 7: [mode, bright, speed, r, g, b, chk]
pub const DEBOUNCE: (u8, u8) = (0x00, 0xA9);
pub const MOTION_SYNC: (u8, u8) = (0x00, 0xAB);
pub const RIPPLE: (u8, u8) = (0x00, 0xB1);
pub const SENSOR_SLEEP_SWITCH: (u8, u8) = (0x00, 0xB5);
pub const SENSOR_SLEEP: (u8, u8) = (0x00, 0xB7);
pub const SENSOR_MODE: (u8, u8) = (0x00, 0xB9);

// Read address aliases mapped to exact flash registers
pub const READ_REPORT_RATE: (u8, u8) = REPORT_RATE;
pub const READ_MAX_DPI_STAGES: (u8, u8) = MAX_DPI_STAGES;
pub const READ_CURRENT_DPI: (u8, u8) = CURRENT_DPI;
pub const READ_LOD: (u8, u8) = LOD;
pub const READ_DEBOUNCE: (u8, u8) = DEBOUNCE;
pub const READ_MOTION_SYNC: (u8, u8) = MOTION_SYNC;
pub const READ_RIPPLE: (u8, u8) = RIPPLE;
pub const READ_ANGLE_SNAP: (u8, u8) = ANGLE_SNAP_X;
pub const ANGLE_SNAP: (u8, u8) = ANGLE_SNAP_X;
pub const READ_SENSOR_SLEEP: (u8, u8) = SENSOR_SLEEP;
pub const READ_SENSOR_MODE: (u8, u8) = SENSOR_MODE;
pub const READ_VERSION: (u8, u8) = (0x00, 0x06);
pub const READ_DPI_LED: (u8, u8) = DPI_LED_COMPOSITE;
pub const READ_BATTERY: (u8, u8) = (0x00, 0x05);

// Legacy aliases
pub const DPI_LED: (u8, u8) = READ_DPI_LED;
pub const BATTERY: (u8, u8) = READ_BATTERY;
pub const VERSION: (u8, u8) = READ_VERSION;
pub const FLASH_ALL: (u8, u8) = (0x00, 0x00);
