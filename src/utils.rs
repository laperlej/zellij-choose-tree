pub struct IdGenerator {
    id: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            id: 0,
        }
    }
    pub fn next(&mut self) -> usize {
        let id = self.id;
        self.id += 1;
        id
    }
}

pub struct KeybindGenerator {
    id: usize,
}

impl KeybindGenerator {
    pub fn new() -> Self {
        Self {
            id: 0,
        }
    }

    pub fn next(&mut self) -> String {
        let id = self.id;
        self.id += 1;
        to_keybind(id)
    }
}

pub fn to_keybind(index: usize) -> String {
    if index >= 36 {
        return ' '.to_string();
    }
    if index < 10 {
        return format!("{}", index);
    }
    let ascii_value = (b'A' as usize + index - 10) as u32;
    char::from_u32(ascii_value).expect("ascii value out of range").to_string()
}
