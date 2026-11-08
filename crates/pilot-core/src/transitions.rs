use crate::types::Mode;

/// Why a mode wants to end. `next_mode` maps these to the next mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Jump,
    LostGround,
    SlideStart,
    SlideStop,
    Land,
    LandSliding,
    WallContact,
    WallLost,
    WallTimeout,
    WallKick,
    WallDetach,
    HangStart,
    HangRelease,
    LedgeGrab,
    MantleDone,
}

pub fn next_mode(from: Mode, trigger: Trigger) -> Option<Mode> {
    use Mode::*;
    use Trigger::*;
    match (from, trigger) {
        (Grounded, Jump | LostGround) => Some(Airborne),
        (Grounded, SlideStart) => Some(Sliding),
        (Sliding, Jump | LostGround) => Some(Airborne),
        (Sliding, SlideStop) => Some(Grounded),
        (Airborne, Land) => Some(Grounded),
        (Airborne, LandSliding) => Some(Sliding),
        (Airborne, WallContact) => Some(Wallrunning),
        (Airborne, HangStart) => Some(WallHanging),
        (Airborne, LedgeGrab) => Some(Mantling),
        (Wallrunning, WallLost | WallTimeout | WallKick | WallDetach) => Some(Airborne),
        (Wallrunning, Land) => Some(Grounded),
        (Wallrunning, LandSliding) => Some(Sliding),
        (Wallrunning, HangStart) => Some(WallHanging),
        (Wallrunning, LedgeGrab) => Some(Mantling),
        (WallHanging, HangRelease | WallKick) => Some(Airborne),
        (Mantling, MantleDone) => Some(Grounded),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Mode::{self, *};
    use Trigger::*;

    const MODES: [Mode; 6] = [Grounded, Sliding, Airborne, Wallrunning, WallHanging, Mantling];
    const TRIGGERS: [Trigger; 15] = [
        Jump, LostGround, SlideStart, SlideStop, Land, LandSliding, WallContact, WallLost,
        WallTimeout, WallKick, WallDetach, HangStart, HangRelease, LedgeGrab, MantleDone,
    ];
    const ALLOWED: [(Mode, Trigger, Mode); 22] = [
        (Grounded, Jump, Airborne),
        (Grounded, LostGround, Airborne),
        (Grounded, SlideStart, Sliding),
        (Sliding, Jump, Airborne),
        (Sliding, LostGround, Airborne),
        (Sliding, SlideStop, Grounded),
        (Airborne, Land, Grounded),
        (Airborne, LandSliding, Sliding),
        (Airborne, WallContact, Wallrunning),
        (Airborne, HangStart, WallHanging),
        (Airborne, LedgeGrab, Mantling),
        (Wallrunning, WallLost, Airborne),
        (Wallrunning, WallTimeout, Airborne),
        (Wallrunning, WallKick, Airborne),
        (Wallrunning, WallDetach, Airborne),
        (Wallrunning, Land, Grounded),
        (Wallrunning, LandSliding, Sliding),
        (Wallrunning, HangStart, WallHanging),
        (Wallrunning, LedgeGrab, Mantling),
        (WallHanging, HangRelease, Airborne),
        (WallHanging, WallKick, Airborne),
        (Mantling, MantleDone, Grounded),
    ];

    #[test]
    fn every_allowed_transition_lands_in_the_listed_mode() {
        for (from, trigger, to) in ALLOWED {
            assert_eq!(next_mode(from, trigger), Some(to), "{from:?} + {trigger:?}");
        }
    }

    #[test]
    fn nothing_outside_the_list_is_a_transition() {
        let mut count = 0;
        for from in MODES {
            for trigger in TRIGGERS {
                if next_mode(from, trigger).is_some() {
                    count += 1;
                    assert!(
                        ALLOWED.iter().any(|(f, tr, _)| *f == from && *tr == trigger),
                        "unexpected transition {from:?} + {trigger:?}"
                    );
                }
            }
        }
        assert_eq!(count, ALLOWED.len());
    }
}
