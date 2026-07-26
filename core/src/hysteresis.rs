#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerState {
    Below,
    Above,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerEvent {
    Entered,
    Exited,
}

pub struct HysteresisGuard {
    enter_threshold: f64,
    exit_threshold: f64,
    state: TriggerState,
}

impl HysteresisGuard {
    pub fn new(enter_threshold: f64, exit_threshold: f64) -> Self {
        Self {
            enter_threshold,
            exit_threshold,
            state: TriggerState::Below,
        }
    }

    pub fn evaluate(&mut self, current_value: f64) -> Option<TriggerEvent> {
        let prev = self.state;
        match prev {
            TriggerState::Below => {
                if current_value >= self.enter_threshold {
                    self.state = TriggerState::Above;
                    Some(TriggerEvent::Entered)
                } else {
                    None
                }
            }
            TriggerState::Above => {
                if current_value <= self.exit_threshold {
                    self.state = TriggerState::Below;
                    Some(TriggerEvent::Exited)
                } else {
                    None
                }
            }
        }
    }

    pub fn current_state(&self) -> TriggerState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hysteresis_enter_exit_cycle() {
        let mut guard = HysteresisGuard::new(5.0, 3.0);
        assert_eq!(guard.current_state(), TriggerState::Below);

        // Below enter threshold — no event
        assert!(guard.evaluate(4.0).is_none());
        assert_eq!(guard.current_state(), TriggerState::Below);

        // Cross enter threshold
        assert_eq!(guard.evaluate(5.0), Some(TriggerEvent::Entered));
        assert_eq!(guard.current_state(), TriggerState::Above);

        // Still above exit threshold — no event
        assert!(guard.evaluate(4.0).is_none());

        // Cross exit threshold
        assert_eq!(guard.evaluate(3.0), Some(TriggerEvent::Exited));
        assert_eq!(guard.current_state(), TriggerState::Below);
    }

    #[test]
    fn test_no_double_fire() {
        let mut guard = HysteresisGuard::new(5.0, 3.0);
        // Enter
        assert_eq!(guard.evaluate(5.0), Some(TriggerEvent::Entered));
        // Staying above — should NOT fire again
        assert_eq!(guard.evaluate(6.0), None);
        assert_eq!(guard.evaluate(7.0), None);
    }
}
