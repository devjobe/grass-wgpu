use std::collections::HashSet;



pub struct Input<T> {
    pressed: HashSet<T>,
    just_pressed: HashSet<T>,
    just_released: HashSet<T>,
}

impl<T> Default for Input<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T: Copy + Eq + std::hash::Hash> Input<T> {

    pub fn activate(&mut self, value: T) {
        self.pressed.insert(value);
        self.just_pressed.insert(value);
    }

    pub fn deactivate(&mut self, value: T) {
        self.pressed.remove(&value);
        self.just_released.insert(value);
    }

    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }


    pub fn pressed(&self, value: T) -> bool {
        return self.pressed.contains(&value);
    }

    #[allow(dead_code)]
    pub fn just_pressed(&self, value: T) -> bool {
        return self.just_pressed.contains(&value);
    }

    #[allow(dead_code)]
    pub fn just_released(&self, value: T) -> bool {
        return self.just_pressed.contains(&value);
    }

}