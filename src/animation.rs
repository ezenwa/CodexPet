use crate::state::PetState;

const IDLE_MS: [u64; 6] = [1680, 660, 660, 840, 840, 1920];

pub fn frame(state: PetState, elapsed_ms: u64) -> usize {
    match state {
        PetState::Idle | PetState::Offline => idle(elapsed_ms),
        PetState::Working => row(7, 6, 120, 220, elapsed_ms),
        PetState::Input => row(6, 6, 150, 260, elapsed_ms),
        PetState::Failed => row(5, 8, 140, 240, elapsed_ms),
        PetState::Ready => {
            let celebration = 5 * 150 + 280;
            if elapsed_ms < celebration * 3 {
                row(8, 6, 150, 280, elapsed_ms)
            } else {
                idle(elapsed_ms - celebration * 3)
            }
        }
    }
}
fn idle(elapsed_ms: u64) -> usize {
    let mut cursor = elapsed_ms % IDLE_MS.iter().sum::<u64>();
    for (i, duration) in IDLE_MS.iter().enumerate() {
        if cursor < *duration {
            return i;
        }
        cursor -= duration;
    }
    0
}
fn row(row: usize, count: usize, duration: u64, final_duration: u64, elapsed: u64) -> usize {
    let time = elapsed % ((count as u64 - 1) * duration + final_duration);
    row * 8 + ((time / duration) as usize).min(count - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_returns_to_idle_after_three_celebrations() {
        assert_eq!(frame(PetState::Ready, 0), 64);
        assert_eq!(frame(PetState::Ready, 3090), 0);
    }
    #[test]
    fn every_state_stays_in_the_sheet() {
        for state in PetState::ALL {
            for t in (0..100_000).step_by(73) {
                assert!(frame(state, t) < 72);
            }
        }
    }
}
