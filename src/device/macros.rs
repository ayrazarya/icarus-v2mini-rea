//! Hardware EEPROM macro programming routines.

use super::IcarusDevice;
use crate::config::MouseConfig;
use crate::protocol::{
    Command, MACRO_SLOT_BASE_ADDR, MACRO_SLOT_SIZE, MAX_MACRO_SLOTS, macro_to_hardware_buffer,
};
use anyhow::{Result, bail};

impl IcarusDevice {
    /// Flash an on-board macro sequence to mouse hardware EEPROM at specific slot (0..15).
    /// Matches chunk transfer decompiled from sub_10005460 and sub_100055b0 in HIDUsb.dll.
    pub fn flash_macro_slot(
        &self,
        slot: u8,
        name: &str,
        events: &[crate::config::MacroEvent],
    ) -> Result<()> {
        if slot >= MAX_MACRO_SLOTS as u8 {
            bail!("Macro slot must be 0..15, got {}", slot);
        }
        if !self.read_device_online()? {
            bail!("Mouse is offline or asleep (2.4G wireless link inactive)");
        }
        let buf = macro_to_hardware_buffer(name, events);
        let base_addr = MACRO_SLOT_BASE_ADDR + (slot as u16) * MACRO_SLOT_SIZE;

        let mut offset = 0usize;
        while offset < buf.len() {
            let chunk_len = (buf.len() - offset).min(10);
            let addr = base_addr + offset as u16;
            let addr_hi = (addr >> 8) as u8;
            let addr_lo = (addr & 0xFF) as u8;
            let cmd = Command::write(addr_hi, addr_lo, &buf[offset..offset + chunk_len]);
            self.send_write(&cmd)?;
            offset += chunk_len;
        }
        Ok(())
    }

    /// Flash all macros from `MouseConfig` into hardware slots and commit any macro button bindings.
    pub fn flash_all_assigned_macros(&self, config: &MouseConfig) -> Result<()> {
        for (slot_idx, macro_item) in config.macros.iter().enumerate().take(MAX_MACRO_SLOTS) {
            self.flash_macro_slot(slot_idx as u8, &macro_item.name, &macro_item.events)?;
        }
        let mut any_macro_btn = false;
        for (i, btn) in config.buttons.iter().enumerate().take(6) {
            if matches!(btn, crate::config::ButtonAction::Macro(_)) {
                self.set_button_mapping(i as u8, btn)?;
                any_macro_btn = true;
            }
        }
        if any_macro_btn {
            self.commit_config(config.active_profile.clamp(1, 5))?;
        }
        Ok(())
    }
}
