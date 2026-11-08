use glam::Vec3;
use pilot_core::{Button, Input};

use crate::UNITS_TO_METRES;

pub const FORMAT_LINE: &str = "# cyberfall-telemetry 1";
pub const COLUMNS: &str =
    "tick,time,px,py,pz,vx,vy,vz,pitch,yaw,fwd,right,jump,crouch,sprint,ground,wallrun,hang,crouched";

/// Columns after the tick index that hold numbers, then the 0/1 flags.
const NUMBER_COUNT: usize = 11;
const FLAG_COUNT: usize = 7;
const FIELD_COUNT: usize = 1 + NUMBER_COUNT + FLAG_COUNT;

/// The movement states Northstar can report. It has no slide or mantle query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoarseMode {
    Ground,
    Air,
    Wallrun,
    Hang,
}

/// One recorded server frame, converted to pilot-core's conventions:
/// metres, radians, pitch positive looking up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tick {
    pub time: f32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub input: Input,
    pub mode: CoarseMode,
    pub crouched: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recording {
    pub name: String,
    pub map: String,
    pub settings: String,
    pub ticks: Vec<Tick>,
}

/// A row exactly as recorded: engine units and degrees.
struct Row {
    /// time, px, py, pz, vx, vy, vz, pitch, yaw, fwd, right
    numbers: [f32; NUMBER_COUNT],
    /// jump, crouch, sprint, ground, wallrun, hang, crouched
    flags: [bool; FLAG_COUNT],
}

fn parse_row(line: &str, line_number: usize) -> Result<Row, String> {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    if fields.len() != FIELD_COUNT {
        return Err(format!(
            "line {line_number}: expected {FIELD_COUNT} fields, found {}",
            fields.len()
        ));
    }
    let mut numbers = [0.0; NUMBER_COUNT];
    for (slot, field) in numbers.iter_mut().zip(&fields[1..=NUMBER_COUNT]) {
        *slot = field
            .parse()
            .map_err(|_| format!("line {line_number}: `{field}` is not a number"))?;
    }
    let mut flags = [false; FLAG_COUNT];
    for (slot, field) in flags.iter_mut().zip(&fields[1 + NUMBER_COUNT..]) {
        *slot = match *field {
            "0" => false,
            "1" => true,
            other => return Err(format!("line {line_number}: flag `{other}` is not 0 or 1")),
        };
    }
    Ok(Row { numbers, flags })
}

fn button(held: bool, previous: Option<bool>) -> Button {
    // A press is a 0 -> 1 change.
    Button { held, pressed: held && previous == Some(false) }
}

fn to_tick(row: &Row, previous: Option<&Row>) -> Tick {
    let n = &row.numbers;
    let [jump, crouch, sprint, ground, wallrun, hang, crouched] = row.flags;
    let before = |index: usize| previous.map(|p| p.flags[index]);
    let mode = if hang {
        CoarseMode::Hang
    } else if wallrun {
        CoarseMode::Wallrun
    } else if ground {
        CoarseMode::Ground
    } else {
        CoarseMode::Air
    };
    Tick {
        time: n[0],
        position: Vec3::new(n[1], n[2], n[3]) * UNITS_TO_METRES,
        velocity: Vec3::new(n[4], n[5], n[6]) * UNITS_TO_METRES,
        input: Input {
            move_forward: n[9],
            move_right: n[10],
            yaw: n[8].to_radians(),
            // The engine reports pitch positive looking down.
            pitch: -n[7].to_radians(),
            jump: button(jump, before(0)),
            crouch: button(crouch, before(1)),
            sprint: button(sprint, before(2)),
            hang: Button::UP,
        },
        mode,
        crouched,
    }
}

impl Recording {
    pub fn parse(text: &str) -> Result<Recording, String> {
        let mut lines = text.lines().enumerate().map(|(i, line)| (i + 1, line.trim_end()));
        match lines.next() {
            Some((_, line)) if line == FORMAT_LINE => {}
            _ => return Err(format!("line 1: expected `{FORMAT_LINE}`")),
        }

        let mut recording = Recording {
            name: String::new(),
            map: String::new(),
            settings: String::new(),
            ticks: Vec::new(),
        };
        let mut saw_columns = false;
        let mut previous: Option<Row> = None;
        for (line_number, line) in lines {
            if line.is_empty() {
                continue;
            }
            if let Some(header) = line.strip_prefix("# ") {
                if let Some((key, value)) = header.split_once(": ") {
                    match key {
                        "name" => recording.name = value.to_string(),
                        "map" => recording.map = value.to_string(),
                        "settings" => recording.settings = value.to_string(),
                        _ => {}
                    }
                }
                continue;
            }
            if !saw_columns {
                if line != COLUMNS {
                    return Err(format!("line {line_number}: unexpected column header"));
                }
                saw_columns = true;
                continue;
            }
            let row = parse_row(line, line_number)?;
            recording.ticks.push(to_tick(&row, previous.as_ref()));
            previous = Some(row);
        }
        if !saw_columns {
            return Err("no column header found".to_string());
        }
        Ok(recording)
    }

    /// The typical time between rows: the median, so a hitch does not skew it.
    pub fn frame_time(&self) -> f32 {
        let mut gaps: Vec<f32> = self.ticks.windows(2).map(|pair| pair[1].time - pair[0].time).collect();
        if gaps.is_empty() {
            return 0.0;
        }
        gaps.sort_by(f32::total_cmp);
        gaps[gaps.len() / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(rows: &[&str]) -> String {
        let mut text = format!("{FORMAT_LINE}\n# map: mp_lf_stacks\n# settings: pilot_mp\n# name: sprint-1\n{COLUMNS}\n");
        for row in rows {
            text.push_str(row);
            text.push('\n');
        }
        text
    }

    //            tick time    px   py   pz  vx  vy vz  pitch yaw fwd right j c s g w h cr
    const STILL: &str = "0,10.0000,100.00,-200.00,50.00,0.00,0.00,0.00,0.00,0.00,0.000,0.000,0,0,0,1,0,0,0";

    #[test]
    fn headers_are_read() {
        let r = Recording::parse(&sample(&[STILL])).unwrap();
        assert_eq!((r.name.as_str(), r.map.as_str(), r.settings.as_str()), ("sprint-1", "mp_lf_stacks", "pilot_mp"));
        assert_eq!(r.ticks.len(), 1);
    }

    #[test]
    fn positions_and_velocities_become_metres() {
        let row = "0,10.0000,100.00,-200.00,50.00,260.00,-10.00,20.00,0.00,0.00,0.000,0.000,0,0,0,1,0,0,0";
        let tick = Recording::parse(&sample(&[row])).unwrap().ticks[0];
        assert!((tick.position - Vec3::new(2.54, -5.08, 1.27)).length() < 1e-5);
        assert!((tick.velocity - Vec3::new(6.604, -0.254, 0.508)).length() < 1e-5);
        assert_eq!(tick.time, 10.0);
    }

    #[test]
    fn angles_become_radians_with_pitch_flipped() {
        let row = "0,10.0000,0.00,0.00,0.00,0.00,0.00,0.00,30.00,90.00,1.000,-0.500,0,0,0,1,0,0,0";
        let tick = Recording::parse(&sample(&[row])).unwrap().ticks[0];
        assert!((tick.input.yaw - 90f32.to_radians()).abs() < 1e-6);
        assert!((tick.input.pitch + 30f32.to_radians()).abs() < 1e-6, "looking down is negative pitch");
        assert_eq!((tick.input.move_forward, tick.input.move_right), (1.0, -0.5));
    }

    #[test]
    fn a_press_is_a_zero_to_one_change_and_never_the_first_row() {
        let down = "0,10.0000,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.000,0.000,1,0,1,1,0,0,0";
        let up = "1,10.0167,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.000,0.000,0,1,1,1,0,0,0";
        let again = "2,10.0333,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.000,0.000,1,1,1,1,0,0,0";
        let ticks = Recording::parse(&sample(&[down, up, again])).unwrap().ticks;
        assert_eq!(ticks[0].input.jump, Button::HELD, "down on the first row is held, not pressed");
        assert_eq!(ticks[1].input.jump, Button::UP);
        assert_eq!(ticks[2].input.jump, Button::PRESSED);
        assert_eq!(ticks[1].input.crouch, Button::PRESSED);
        assert_eq!(ticks[2].input.crouch, Button::HELD);
        assert_eq!(ticks[2].input.sprint, Button::HELD);
    }

    #[test]
    fn coarse_mode_prefers_hang_then_wallrun_then_ground() {
        let flags = |g: u8, w: u8, h: u8| {
            format!("0,10.0000,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.000,0.000,0,0,0,{g},{w},{h},1")
        };
        let mode = |g, w, h| Recording::parse(&sample(&[&flags(g, w, h)])).unwrap().ticks[0].mode;
        assert_eq!(mode(1, 0, 0), CoarseMode::Ground);
        assert_eq!(mode(0, 0, 0), CoarseMode::Air);
        assert_eq!(mode(0, 1, 0), CoarseMode::Wallrun);
        assert_eq!(mode(1, 1, 0), CoarseMode::Wallrun);
        assert_eq!(mode(1, 1, 1), CoarseMode::Hang);
        assert!(Recording::parse(&sample(&[&flags(1, 0, 0)])).unwrap().ticks[0].crouched);
    }

    #[test]
    fn frame_time_is_the_median_gap() {
        let at = |tick: u32, time: &str| {
            format!("{tick},{time},0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.000,0.000,0,0,0,1,0,0,0")
        };
        let rows = [at(0, "10.0000"), at(1, "10.0200"), at(2, "10.0400"), at(3, "10.5400"), at(4, "10.5600")];
        let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        let r = Recording::parse(&sample(&refs)).unwrap();
        assert!((r.frame_time() - 0.02).abs() < 1e-4, "a half-second hitch must not skew it: {}", r.frame_time());
    }

    #[test]
    fn a_wrong_format_line_is_rejected() {
        let text = sample(&[STILL]).replace("telemetry 1", "telemetry 2");
        assert!(Recording::parse(&text).unwrap_err().contains("line 1"));
    }

    #[test]
    fn a_wrong_column_header_is_rejected() {
        let text = sample(&[STILL]).replace(",hang,", ",hung,");
        assert!(Recording::parse(&text).unwrap_err().contains("column header"));
    }

    #[test]
    fn malformed_rows_name_their_line() {
        let short = "1,10.0167,0.00,0.00";
        let error = Recording::parse(&sample(&[STILL, short])).unwrap_err();
        assert!(error.contains("line 7") && error.contains("fields"), "{error}");

        let bad_number = STILL.replace("100.00", "abc");
        let error = Recording::parse(&sample(&[&bad_number])).unwrap_err();
        assert!(error.contains("line 6") && error.contains("abc"), "{error}");

        let bad_flag = "0,10.0000,100.00,-200.00,50.00,0.00,0.00,0.00,0.00,0.00,0.000,0.000,0,0,0,2,0,0,0";
        let error = Recording::parse(&sample(&[bad_flag])).unwrap_err();
        assert!(error.contains("line 6") && error.contains("flag"), "{error}");
    }
}
