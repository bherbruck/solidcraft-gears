mod gear;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

wit_bindgen::generate!({
    world: "command-plugin",
});

use craft::plugin::commands;
use exports::craft::plugin::entry::{Guest, PluginError};

const INCH_TO_MM: f64 = 25.4;
const PRINT_CLEARANCE_MM: f64 = 6.35; // 0.25 in gap for Separated print layout

#[derive(Deserialize)]
struct InputParams {
    #[serde(default = "default_units")]
    units: String,
    module: f64,
    teeth: u32,
    face_width: f64,
    #[serde(default = "default_pressure_angle")]
    pressure_angle: f64,
    #[serde(default)]
    bore_diameter: f64,
    #[serde(default)]
    profile_shift: f64,
    #[serde(default)]
    mate_gear_index: usize,
    #[serde(default = "default_placement_mode")]
    placement_mode: String,
    #[serde(default = "default_mate_side")]
    mate_side: String,
    #[serde(default)]
    x_offset: f64,
    #[serde(default)]
    y_offset: f64,
}

fn default_units() -> String {
    String::new()
    }
fn default_pressure_angle() -> f64 {
    20.0
}
fn default_placement_mode() -> String {
    "separated".into()
}
fn default_mate_side() -> String {
    "right".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExistingGear {
    number: usize,
    name: String,
    cx_mm: f64,
    cy_mm: f64,
    pitch_d_mm: f64,
    outer_d_mm: f64,
    teeth: u32,
    phase_deg: f64,
}

fn run_cmd(command: &str, args: Value) -> Result<Value, PluginError> {
    let raw = commands::run(command, &args.to_string())?;
    serde_json::from_str(&raw).map_err(|e| {
        PluginError::Trap(format!("Failed to parse reply from {command}: {e}"))
    })
}

fn err_input(field: &str, reason: &str) -> PluginError {
    PluginError::InvalidInput(craft::plugin::types::FieldFailure {
        field: field.to_string(),
        reason: reason.to_string(),
    })
}

fn find_existing_gears() -> Vec<ExistingGear> {
    let Ok(doc) = run_cmd(
        "document.inspect",
        json!({ "measure": false, "timeline": false, "bodies": "names" }),
    ) else {
        return Vec::new();
    };

    let mut gears = Vec::new();
    let Some(bodies) = doc.get("bodies").and_then(|b| b.as_array()) else {
        return gears;
    };

    for entry in bodies {
        let name_opt = entry
            .as_str()
            .or_else(|| entry.get("name").and_then(|n| n.as_str()));
        let Some(name) = name_opt else {
            continue;
        };
        if !name.starts_with("Gear ") {
            continue;
        }
        let after_prefix = &name[5..];
        let num_str: String = after_prefix.chars().take_while(|c| c.is_ascii_digit()).collect();
        let Ok(number) = num_str.parse::<usize>() else {
            continue;
        };

        let mut teeth = 20u32;
        let mut pitch_d_mm = 0.0f64;
        let mut outer_d_mm = 0.0f64;
        let mut cx_mm = 0.0f64;
        let mut cy_mm = 0.0f64;
        let mut phase_deg = 0.0f64;

        if let Some(start) = name.find('[') {
            if let Some(end) = name[start..].find(']') {
                let meta = &name[start + 1..start + end];
                for part in meta.split(',') {
                    if let Some((k, v)) = part.split_once('=') {
                        match k.trim() {
                            "z" => teeth = v.trim().parse().unwrap_or(teeth),
                            "pd" => pitch_d_mm = v.trim().parse().unwrap_or(pitch_d_mm),
                            "od" => outer_d_mm = v.trim().parse().unwrap_or(outer_d_mm),
                            "cx" => cx_mm = v.trim().parse().unwrap_or(cx_mm),
                            "cy" => cy_mm = v.trim().parse().unwrap_or(cy_mm),
                            "ph" => phase_deg = v.trim().parse().unwrap_or(phase_deg),
                            _ => {}
                        }
                    }
                }
            }
        }

        gears.push(ExistingGear {
            number,
            name: name.to_string(),
            cx_mm,
            cy_mm,
            pitch_d_mm,
            outer_d_mm,
            teeth,
            phase_deg,
        });
    }

    gears.sort_by_key(|g| g.number);
    gears
}

fn compute_meshing_phase_deg(target: &ExistingGear, new_teeth: u32, dir_angle_deg: f64) -> f64 {
    let target_pitch_deg = 360.0 / (target.teeth as f64);
    let new_pitch_deg = 360.0 / (new_teeth as f64);

    let rel_target_deg = dir_angle_deg - target.phase_deg;
    let target_tooth_phase = (rel_target_deg / target_pitch_deg).rem_euclid(1.0);

    let opposite_dir_deg = dir_angle_deg + 180.0;
    let desired_new_phase = (0.5 - target_tooth_phase).rem_euclid(1.0);
    let raw_phase = opposite_dir_deg - desired_new_phase * new_pitch_deg;
    raw_phase.rem_euclid(360.0)
}

struct Component;

impl Guest for Component {
    fn execute(entry: String, params: String) -> Result<String, PluginError> {
        if entry != "make-gear-in" && entry != "make-gear-mm" && entry != "make-gear" {
            return Err(err_input("entry", &format!("unknown entry: {entry}")));
        }
let input: InputParams = serde_json::from_str(&params)
            .map_err(|e| err_input("params", &format!("invalid JSON: {e}")))?;

        let is_inch = if entry.ends_with("-mm") || entry.ends_with("_mm") {
            false
        } else if entry.ends_with("-in") || entry.ends_with("_in") {
            true
        } else if input.units.is_empty() {
            entry != "make-gear-mm"
        } else {
            input.units.eq_ignore_ascii_case("in")
        };
 let input: InputParams = serde_json::from_str(&params)
            .map_err(|e| err_input("params", &format!("invalid JSON: {e}")))?;

        let is_inch = if entry.ends_with("-mm") || entry.ends_with("_mm") {
            false
        } else if entry.ends_with("-in") || entry.ends_with("_in") {
            true
        } else if input.units.is_empty() {
            entry != "make-gear-mm"
        } else {
            input.units.eq_ignore_ascii_case("in")
        };
        let scale = if is_inch { INCH_TO_MM } else { 1.0 };
         let gp = gear::Params {
            module: input.module * scale,
            teeth: input.teeth,
            pressure_angle_deg: input.pressure_angle,
            face_width: input.face_width * scale,
            bore: input.bore_diameter * scale,
            shift: input.profile_shift,
        };

        gear::validate(&gp).map_err(|(f, r)| err_input(&f, &r))?;
        let dims = gear::dims(&gp);
        let wedge = gear::wedge(&gp).map_err(|r| err_input("profile_shift", &r))?;

        let existing_gears = find_existing_gears();
        let next_num = existing_gears
            .iter()
            .map(|g| g.number)
            .max()
            .unwrap_or(0)
            + 1;

        let resolved_target: Option<&ExistingGear> = if input.mate_gear_index > 0 {
            let found = existing_gears.iter().find(|g| g.number == input.mate_gear_index);
            if found.is_none() {
                return Err(err_input(
                    "mate_gear_index",
                    &format!("Gear {} was not found in the active design", input.mate_gear_index),
                ));
            }
            found
        } else {
            None
        };

        let (target_x_mm, target_y_mm, phase_deg) = if let Some(target) = resolved_target {
            let (ux, uy, dir_deg) = match input.mate_side.as_str() {
                "left" => (-1.0, 0.0, 180.0),
                "top" => (0.0, 1.0, 90.0),
                "bottom" => (0.0, -1.0, 270.0),
                _ => (1.0, 0.0, 0.0),
            };

            let center_dist_mm = if input.placement_mode == "meshed" {
                0.5 * (target.pitch_d_mm + dims.pitch_d)
            } else {
                0.5 * (target.outer_d_mm + dims.outer_d) + PRINT_CLEARANCE_MM
            };

            let cx = target.cx_mm + ux * center_dist_mm;
            let cy = target.cy_mm + uy * center_dist_mm;
            let phase = compute_meshing_phase_deg(target, gp.teeth, dir_deg);
            (cx, cy, phase)
        } else {
            (input.x_offset * scale, input.y_offset * scale, 0.0)
        };

        let unit_label = if is_inch { "in" } else { "mm" };
        let outer_disp = dims.outer_d / scale;
        let pitch_disp = dims.pitch_d / scale;

        let body_name = format!(
            "Gear {} ({}T, OD {:.3} {}) [z={},pd={:.4},od={:.4},cx={:.4},cy={:.4},ph={:.4}]",
            next_num,
            gp.teeth,
            outer_disp,
            unit_label,
            gp.teeth,
            dims.pitch_d,
            dims.outer_d,
            target_x_mm,
            target_y_mm,
            phase_deg
        );

        let hub_radius = 0.5 * (wedge.inner_radius + (dims.root_d / 2.0));
        run_cmd(
            "solid.cylinder",
            json!({
                "radius": hub_radius,
                "height": gp.face_width,
                "base": [0.0, 0.0, 0.0],
                "axis": "Z",
                "operation": "new",
                "name": format!("Gear_{}_Hub", next_num),
                "body_name": body_name
            }),
        )?;

        let sketch_name = format!("Gear_{}_Tooth_Sketch", next_num);
        run_cmd(
            "sketch.create",
            json!({
                "plane": "XY",
                "name": sketch_name
            }),
        )?;

        for seg in &wedge.segs {
            match seg {
                gear::Seg::Line(a, b) => {
                    run_cmd(
                        "sketch.line",
                        json!({
                            "points": [a, b],
                            "connect": true
                        }),
                    )?;
                }
                gear::Seg::Arc(a, b, mid) => {
                    run_cmd(
                        "sketch.arc.three_point",
                        json!({
                            "start": a,
                            "end": b,
                            "through": mid,
                            "connect": true
                        }),
                    )?;
                }
                gear::Seg::Spline(pts) => {
                    run_cmd(
                        "sketch.spline.fit_point",
                        json!({
                            "points": pts,
                            "periodic": false
                        }),
                    )?;
                }
            }
        }

        run_cmd("sketch.finish", json!({}))?;

        let tooth_feat_name = format!("Gear_{}_Tooth", next_num);
        run_cmd(
            "solid.extrude",
            json!({
                "sketch": sketch_name,
                "distance": gp.face_width,
                "direction": "positive",
                "operation": "join",
                "targets": [body_name],
                "name": tooth_feat_name
            }),
        )?;

        run_cmd(
            "solid.pattern.circular",
            json!({
                "features": [tooth_feat_name],
                "axis": "Z",
                "count": gp.teeth,
                "angle": 360.0
            }),
        )?;

        if gp.bore > 0.0 {
            run_cmd(
                "solid.cylinder",
                json!({
                    "diameter": gp.bore,
                    "height": gp.face_width,
                    "base": [0.0, 0.0, 0.0],
                    "axis": "Z",
                    "operation": "cut",
                    "targets": [body_name],
                    "name": format!("Gear_{}_Bore", next_num)
                }),
            )?;
        }

        if phase_deg.abs() > 1e-6 || target_x_mm.abs() > 1e-6 || target_y_mm.abs() > 1e-6 {
            run_cmd(
                "solid.move",
                json!({
                    "bodies": [body_name],
                    "origin": [0.0, 0.0, 0.0],
                    "axis": [0.0, 0.0, 1.0],
                    "angle": phase_deg,
                    "translate": [target_x_mm, target_y_mm, 0.0]
                }),
            )?;
        }

        let summary = json!({
            "gear": body_name,
            "units": unit_label,
            "teeth": gp.teeth,
            "pitch_diameter": format!("{:.4} {}", pitch_disp, unit_label),
            "outer_diameter": format!("{:.4} {}", outer_disp, unit_label),
            "phase_angle_deg": format!("{:.3}", phase_deg),
            "center_x": format!("{:.4} {}", target_x_mm / scale, unit_label),
            "center_y": format!("{:.4} {}", target_y_mm / scale, unit_label)
        });

        Ok(summary.to_string())
    }
}

export!(Component);