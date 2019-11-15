/// A timer that ticks down. This is used for the 60hz sound and delay timers by [`CPU`].
#[derive(Debug)]
pub struct Timer {
    value: u8,
}

impl Timer {
    fn new() -> Self {
        Self { value: 0 }
    }

    pub fn current_value(&self) -> u8 {
        self.value
    }

    pub fn set_value(&mut self, new_value: u8) {
        self.value = new_value;
    }

    pub fn tick(&mut self) {
        if self.is_active() {
            self.value -= 1;
        }
    }

    pub fn is_active(&self) -> bool {
        self.value > 0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Timer;

    #[test]
    fn test_default() {
        let t = Timer::default();

        assert_eq!(t.is_active(), false);
        assert_eq!(t.current_value(), 0);
    }

    #[test]
    fn test_tick_when_value_is_zero() {
        let mut t = Timer::default();

        t.tick();

        assert_eq!(t.is_active(), false);
        assert_eq!(t.current_value(), 0);
    }

    #[test]
    fn test_tick_when_value_is_non_zero() {
        let mut t = Timer::default();
        t.set_value(2);

        t.tick();
        assert_eq!(t.is_active(), true);
        assert_eq!(t.current_value(), 1);

        t.tick();
        assert_eq!(t.is_active(), false);
        assert_eq!(t.current_value(), 0);
    }
}
