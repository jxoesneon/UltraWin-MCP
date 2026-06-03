use crate::traits::InputProvider;
use anyhow::Result;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub struct InputManager {}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputProvider for InputManager {
    fn mouse_click(&self, x: i32, y: i32, button: &str) -> Result<()> {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            let inputs = generate_mouse_click_inputs(x, y, button, width, height);
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            Ok(())
        }
    }

    fn type_text(&self, text: &str) -> Result<()> {
        unsafe {
            for c in text.chars() {
                let inputs = generate_key_inputs(c);
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
            Ok(())
        }
    }
}

pub fn generate_mouse_click_inputs(
    x: i32,
    y: i32,
    button: &str,
    width: i32,
    height: i32,
) -> [INPUT; 3] {
    let normalized_x = if width > 0 { (x * 65535) / width } else { 0 };
    let normalized_y = if height > 0 { (y * 65535) / height } else { 0 };

    let mut inputs = [INPUT::default(); 3];

    inputs[0].r#type = INPUT_MOUSE;
    inputs[0].Anonymous.mi = MOUSEINPUT {
        dx: normalized_x,
        dy: normalized_y,
        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
        ..Default::default()
    };

    let (down, up) = match button {
        "right" => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        "middle" => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
    };

    inputs[1].r#type = INPUT_MOUSE;
    inputs[1].Anonymous.mi = MOUSEINPUT {
        dwFlags: MOUSEEVENTF_ABSOLUTE | down,
        ..Default::default()
    };

    inputs[2].r#type = INPUT_MOUSE;
    inputs[2].Anonymous.mi = MOUSEINPUT {
        dwFlags: MOUSEEVENTF_ABSOLUTE | up,
        ..Default::default()
    };

    inputs
}

pub fn generate_key_inputs(c: char) -> [INPUT; 2] {
    let mut inputs = [INPUT::default(); 2];

    inputs[0].r#type = INPUT_KEYBOARD;
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wScan: c as u16,
        dwFlags: KEYEVENTF_UNICODE,
        ..Default::default()
    };

    inputs[1].r#type = INPUT_KEYBOARD;
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wScan: c as u16,
        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
        ..Default::default()
    };

    inputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_logic() {
        let inputs = generate_mouse_click_inputs(10, 10, "left", 100, 100);
        assert_eq!(inputs.len(), 3);

        let keys = generate_key_inputs('a');
        assert_eq!(keys.len(), 2);
    }
}

