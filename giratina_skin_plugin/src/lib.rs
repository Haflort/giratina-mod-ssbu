#![feature(proc_macro_hygiene)]
// Giratina for Ridley (c04/c05) - Skyline plugin for Super Smash Bros. Ultimate.
// Written by haflort with AI assistance from Claude (Anthropic): the code was generated, debugged and
// refactored with Claude and reviewed/tested in game by the author.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]
// RELEASE build - Giratina (Ridley c04/c05): Altered <-> Origin Forme. Visual only (no stat, speed or armor changes).
// Every match starts in Altered Forme. Down taunt switches form, the Final Smash turns him into Origin Forme.
// Origin Forme = Origin meshes shown, Altered meshes hidden, and neck/body/tail joints stretched.

use smash::app::lua_bind::*;
use smash::app::BattleObjectModuleAccessor;
use smash::lib::lua_const::*;
use smash::lua2cpp::L2CFighterCommon;
use smash::phx::{Hash40, Vector3f};

const SLOTS: [i32; 2] = [4, 5];
const ALTERED: [&str; 3] = ["zz_gir_bodya", "zz_gir_bodyb", "zz_gir_eye"];
const ORIGIN: [&str; 3] = ["zz_gir_obodya", "zz_gir_obodyb", "zz_gir_oeye"];
// stretched local translations (Ridley model units), generated from gir/origin_stretch.json
const STRETCH: [(&str, [f32; 3]); 13] = [
    ("Bust", [5.6492, 0.0, 0.0]),
    ("Neck", [6.3873, 0.0, 0.0]),
    ("Neck1", [6.9678, 0.0, -0.6497]),
    ("Head", [6.2335, 0.0, 0.0]),
    ("Tail", [4.116, 0.0, 1.9826]),
    ("Tail1", [2.7644, 0.0, 0.0]),
    ("Tail2", [2.7499, 0.0, 0.283]),
    ("Tail3", [2.4704, 0.0, 0.0]),
    ("Tail4", [2.4534, 0.0, 0.2896]),
    ("Tail5", [2.3653, 0.0, 0.0]),
    ("Tail6", [2.3579, 0.0, 0.1863]),
    ("Tail7", [2.1941, 0.0, 0.0]),
    ("Tail8", [2.1862, 0.0, 0.1859]),
];

static mut IS_ORIGIN: [bool; 8] = [false; 8];
static mut NEED_CLEAR: [bool; 8] = [false; 8];
static mut PREV_STATUS: [i32; 8] = [-1; 8];
static mut TAUNT_SWITCHED: [bool; 8] = [false; 8];
const TAUNT_SWITCH_FRAME: f32 = 8.0;     // v14: swapped while he is swallowed by darkness (Legends Arceus style)

pub fn dlog(_s: &str) {}   // release: no log file

unsafe fn apply_form(boma: *mut BattleObjectModuleAccessor, id: usize) {
    let origin = IS_ORIGIN[id];
    let gone = VANISHED[id];
    for m in ALTERED.iter() { ModelModule::set_mesh_visibility(boma, Hash40::new(m), !origin && !gone); }
    for m in ORIGIN.iter() { ModelModule::set_mesh_visibility(boma, Hash40::new(m), origin && !gone); }
    ModelModule::set_mesh_visibility(boma, Hash40::new("zz_gir_rift"), RIFT_K[id] > 0.0);
    if origin {
        for (b, t) in STRETCH.iter() {
            let v = Vector3f { x: t[0], y: t[1], z: t[2] };
            ModelModule::set_joint_translate(boma, Hash40::new(b), &v, false, false);
        }
    } else if NEED_CLEAR[id] {
        for (b, _) in STRETCH.iter() { ModelModule::clear_joint_srt(boma, Hash40::new(b)); }
        NEED_CLEAR[id] = false;
    }
    rift_pose(boma, id);
}

/// entry id when this object is a Giratina slot (Ridley c08/c09)
unsafe fn giratina_id(boma: *mut BattleObjectModuleAccessor) -> Option<usize> {
    if boma.is_null() { return None; }
    if smash::app::utility::get_category(&mut *boma) != *BATTLE_OBJECT_CATEGORY_FIGHTER { return None; }
    if smash::app::utility::get_kind(&mut *boma) != *FIGHTER_KIND_RIDLEY { return None; }
    let color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
    if !SLOTS.contains(&color) { return None; }
    let id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    if id >= 8 { None } else { Some(id) }
}

static mut FIRST_FRAME_LOGGED: bool = false;
static mut APPEAL_LOGS: u32 = 0;

unsafe fn giratina_frame(fighter: *mut L2CFighterCommon) {
    let boma = (&*fighter).module_accessor;
    let id = match giratina_id(boma) { Some(i) => i, None => return };
    if !FIRST_FRAME_LOGGED { FIRST_FRAME_LOGGED = true; dlog(&format!("first Giratina frame, entry {}", id)); }
    let status = StatusModule::status_kind(boma);
    if status != PREV_STATUS[id] {
        if status == *FIGHTER_STATUS_KIND_ENTRY {
            if IS_ORIGIN[id] { NEED_CLEAR[id] = true; }
            IS_ORIGIN[id] = false;                                   // every match starts in Altered Forme
        } else if status == *FIGHTER_STATUS_KIND_FINAL {
            IS_ORIGIN[id] = true;                                    // Final Smash -> Origin Forme (stays)
            dlog(&format!("entry {} final smash -> origin", id));
        }
        TAUNT_SWITCHED[id] = false;
        PREV_STATUS[id] = status;
    }
    // down taunt switches form, part-way into the taunt (the roar starts here)
    if status == *FIGHTER_STATUS_KIND_APPEAL && !TAUNT_SWITCHED[id] {
        let mk = MotionModule::motion_kind(boma);
        let frame = MotionModule::frame(boma);
        if APPEAL_LOGS < 4 && frame >= 1.0 {
            APPEAL_LOGS += 1;
            dlog(&format!("entry {} taunt motion {:#x} frame {} (appeal_lw_r = {:#x})", id, mk, frame, smash::hash40("appeal_lw_r")));
        }
        if (mk == smash::hash40("appeal_lw_l") || mk == smash::hash40("appeal_lw_r")) && frame >= TAUNT_SWITCH_FRAME {
            IS_ORIGIN[id] = !IS_ORIGIN[id];
            if !IS_ORIGIN[id] { NEED_CLEAR[id] = true; }
            TAUNT_SWITCHED[id] = true;
            tinted(boma, "sys_whirlwind_r", "top", (0.0, 0.0, 0.0), 1.3, false);
            dlog(&format!("entry {} down taunt -> origin={}", id, IS_ORIGIN[id]));
        }
    }
    if status == *FIGHTER_STATUS_KIND_APPEAL {
        let mk = MotionModule::motion_kind(boma);
        if mk == smash::hash40("appeal_lw_l") || mk == smash::hash40("appeal_lw_r") {
            taunt_fx(boma, id, MotionModule::frame(boma));
        } else { taunt_fx_end(boma, id); }
    } else { taunt_fx_end(boma, id); }
    apply_form(boma, id);
    if MIST_T[id] >= 0 {
        MIST_T[id] += 1;
        if MIST_T[id] > 45 || status != *FIGHTER_STATUS_KIND_APPEAL {
            EffectModule::kill_kind(boma, Hash40::new("sys_damage_purple"), false, true);
            MIST_T[id] = -1;
        }
    }
    fs_cine(boma, id);
}



// dark purple mist bursting out of Giratina when he changes form (common smoke effects, tinted)
unsafe fn tinted(boma: *mut BattleObjectModuleAccessor, eff: &str, joint: &str, off: (f32, f32, f32), size: f32, follow: bool) {
    let pos = Vector3f { x: off.0, y: off.1, z: off.2 };
    let rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = if follow {
        EffectModule::req_follow(boma, Hash40::new(eff), Hash40::new(joint), &pos, &rot, size, true, 0, 0, 0, 0, 0, false, false)
    } else {
        let rnd = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        EffectModule::req_on_joint(boma, Hash40::new(eff), Hash40::new(joint), &pos, &rot, size, &rnd, &rnd, false, 0, 0, 0)
    } as u32;
    if h != 0 {
        EffectModule::set_rgb(boma, h, 0.22, 0.02, 0.32);
    }
}

static mut MIST_T: [i32; 8] = [-1; 8];
unsafe fn shadow_mist(boma: *mut BattleObjectModuleAccessor) {
    for (j, o, sz) in [("hip", (0.0, 0.0, 0.0), 2.6), ("bust", (0.0, 0.0, 0.0), 2.4), ("head", (0.0, 0.0, 0.0), 2.0),
                        ("tail4", (0.0, 0.0, 0.0), 2.2), ("top", (0.0, 4.0, 0.0), 3.0)].iter() {
        tinted(boma, "sys_atk_smoke", j, *o, *sz, false);
    }
    tinted(boma, "sys_down_smoke", "top", (0.0, 0.0, 0.0), 2.4, false);
    tinted(boma, "sys_whirlwind_r", "top", (0.0, 0.0, 0.0), 1.2, false);
    tinted(boma, "sys_damage_purple", "hip", (0.0, 0.0, 0.0), 2.0, true);
    tinted(boma, "sys_damage_purple", "head", (0.0, 0.0, 0.0), 1.6, true);
}

// ---- v14: down-taunt form change, after Legends: Arceus (Giratina flickers and sheds black smoke, darkness swallows
// him, red eyes glint in the dark, the new form bursts out of it and roars) ----
static mut TAUNT_FX_F: [i32; 8] = [-1; 8];      // last handled frame of the form-change taunt (-1 = not in it)
static mut DARK_ON: [bool; 8] = [false; 8];

unsafe fn tinted_rgb(boma: *mut BattleObjectModuleAccessor, eff: &str, joint: &str, off: (f32, f32, f32), size: f32,
                     follow: bool, rgb: (f32, f32, f32)) {
    let pos = Vector3f { x: off.0, y: off.1, z: off.2 };
    let rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let h = if follow {
        EffectModule::req_follow(boma, Hash40::new(eff), Hash40::new(joint), &pos, &rot, size, true, 0, 0, 0, 0, 0, false, false)
    } else {
        let rnd = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        EffectModule::req_on_joint(boma, Hash40::new(eff), Hash40::new(joint), &pos, &rot, size, &rnd, &rnd, false, 0, 0, 0)
    } as u32;
    if h != 0 { EffectModule::set_rgb(boma, h, rgb.0, rgb.1, rgb.2); }
}

fn dark_level(f: f32) -> f32 {
    if f < 2.0 { 0.0 } else if f < 6.0 { (f - 2.0) / 4.0 } else if f <= 9.0 { 1.0 }
    else if f < 18.0 { 1.0 - (f - 9.0) / 9.0 } else { 0.0 }
}

unsafe fn taunt_fx(boma: *mut BattleObjectModuleAccessor, id: usize, frame: f32) {
    let f = frame as i32;
    if f == TAUNT_FX_F[id] { return; }
    let first = TAUNT_FX_F[id] < 0;
    TAUNT_FX_F[id] = f;
    if first && f > 3 { return; }                        // joined late (e.g. a cancelled taunt): no effects
    // flicker: black smoke shed from different parts of the body
    if f >= 1 && f <= 7 && f % 2 == 1 {
        let js = ["tail2", "bust", "tail6", "hip"];
        let j = js[((f / 2) as usize) % js.len()];
        tinted_rgb(boma, "sys_atk_smoke", j, (0.0, 0.0, 0.0), 1.5, false, (0.03, 0.0, 0.06));
    }
    if f == 2 { shadow_mist(boma); MIST_T[id] = 0; }        // the darkness swallows him
    // the body goes black while the smoke covers him, then the new form comes out of it
    let k = dark_level(frame);
    if k > 0.0 {
        let col = smash::phx::Vector4f { x: 0.02, y: 0.0, z: 0.05, w: k * 0.92 };
        ColorBlendModule::set_main_color(boma, &col, &col, 1.0, 1.0, 0, true);
        DARK_ON[id] = true;
    } else if DARK_ON[id] {
        ColorBlendModule::cancel_main_color(boma, 0);
        DARK_ON[id] = false;
    }
    if f == 7 { tinted_rgb(boma, "sys_smash_flash", "head", (0.0, 0.0, 0.0), 1.0, true, (1.0, 0.05, 0.05)); }   // red eyes in the dark
    if f == 12 {                                                  // bursts out and roars
        tinted_rgb(boma, "sys_crown", "top", (0.0, 0.0, 0.0), 1.1, false, (0.30, 0.02, 0.55));
        tinted_rgb(boma, "sys_down_smoke", "top", (0.0, 0.0, 0.0), 1.8, false, (0.05, 0.0, 0.09));
        CameraModule::req_quake(boma, *CAMERA_QUAKE_KIND_M);
    }
}

unsafe fn taunt_fx_end(boma: *mut BattleObjectModuleAccessor, id: usize) {
    if TAUNT_FX_F[id] < 0 { return; }
    TAUNT_FX_F[id] = -1;
    if DARK_ON[id] { ColorBlendModule::cancel_main_color(boma, 0); DARK_ON[id] = false; }
}

// ---- v12: Distortion World Final Smash ----
// Only the fighter's own module accessor is used here (the cutscene itself lives in the gunship model/motions/cameras).
// final_start: Giratina roars, then vanishes and a rift in reality tears open where he stood (rift = zz_gir_rift mesh on
// the "Throw" joint, held at the vanish point while the invisible dash travels). Cutscene: fighter hidden, Ridley's own
// beams and sounds removed, cry when the cutscene Giratina roars, slash sounds for the claw marks, and every other fighter
// is turned into a white silhouette (they do that themselves through the all-fighter callback below).
static mut VANISHED: [bool; 8] = [false; 8];
static mut AURA_DONE: [bool; 8] = [false; 8];
static mut RIFT_K: [f32; 8] = [0.0; 8];          // rift size (0 = off)
static mut RIFT_T: [i32; 8] = [-1; 8];           // frames since the rift opened (-1 = closed)
static mut RIFT_CLOSING: [i32; 8] = [-1; 8];
static mut RIFT_AT: [(f32, f32); 8] = [(0.0, 0.0); 8];
static mut HIDDEN: [bool; 8] = [false; 8];
static mut CRY_DONE: [bool; 8] = [false; 8];
static mut SLASH_DONE: [u8; 8] = [0; 8];
static mut LAST_MK: [u64; 8] = [0; 8];
static mut MK_LOGS: u32 = 0;
pub static mut CINE: bool = false;               // a Giratina cutscene is running (opponents stand up)
pub static mut WHITE: f32 = 0.0;                 // >0 while the black/white scene runs (read by every fighter)
pub static mut WHITE_FROM: i32 = -1;             // entry id of the Giratina that started it
static mut TINTED: [bool; 8] = [false; 8];

const FINAL_VICTIM_STATUS: i32 = 365;   // status of an opponent caught in Ridley's cutscene (recorded in game)
const VANISH_FRAME: f32 = 30.0;      // final_start frame where Giratina disappears
const RIFT_HEIGHT: f32 = 22.0;       // rift centre above his feet
const RIFT_OPEN: i32 = 10;           // frames to tear fully open
const CRY_FRAME: f32 = 2.0;          // scene 4 frame where the cutscene Giratina roars
const SLASH_FRAMES: [f32; 3] = [30.0, 33.0, 36.0];   // scene 5 claw strokes
const KILL_FX: [&str; 6] = ["ridley_final_attack", "ridley_final_hold", "ridley_final_beam_a", "ridley_final_beam_b",
                            "ridley_final_beam_c", "ridley_final_beam"];
const STOP_SE: [&str; 6] = ["se_ridley_final02", "se_ridley_final03", "se_ridley_final04", "se_ridley_final05",
                            "se_ridley_final06", "se_ridley_final07"];

fn mk_in(mk: u64, names: &[&str]) -> bool { names.iter().any(|n| mk == smash::hash40(n)) }

unsafe fn rift_pose(boma: *mut BattleObjectModuleAccessor, id: usize) {
    if RIFT_K[id] <= 0.0 { return; }
    let p = *PostureModule::pos(boma);
    let lr = PostureModule::lr(boma);
    let mut sc = PostureModule::scale(boma); if sc.abs() < 0.01 { sc = 1.0; }
    let (ax, ay) = RIFT_AT[id];
    // Throw sits under Rot (0, 9.5, 0); model +Z points along the facing direction (world +X * lr)
    let t = Vector3f { x: 0.0, y: (ay + RIFT_HEIGHT - p.y) / sc - 9.5, z: lr * (ax - p.x) / sc };
    ModelModule::set_joint_translate(boma, Hash40::new("Throw"), &t, false, false);
    let k = RIFT_K[id];
    ModelModule::set_joint_scale(boma, Hash40::new("Throw"), &Vector3f { x: k, y: k, z: k });
}

unsafe fn rift_off(boma: *mut BattleObjectModuleAccessor, id: usize) {
    if RIFT_T[id] >= 0 || RIFT_K[id] > 0.0 {
        RIFT_T[id] = -1; RIFT_K[id] = 0.0; RIFT_CLOSING[id] = -1;
        ModelModule::set_mesh_visibility(boma, Hash40::new("zz_gir_rift"), false);
        ModelModule::clear_joint_srt(boma, Hash40::new("Throw"));
    }
}

unsafe fn silence(boma: *mut BattleObjectModuleAccessor) {
    for e in KILL_FX.iter() { EffectModule::kill_kind(boma, Hash40::new(e), false, true); }
    for s in STOP_SE.iter() { SoundModule::stop_se(boma, Hash40::new(s), 0); }
}

unsafe fn fs_cine(boma: *mut BattleObjectModuleAccessor, id: usize) {
    let mk = MotionModule::motion_kind(boma);
    let f = MotionModule::frame(boma);
    if mk != LAST_MK[id] {
        if MK_LOGS < 40 && (LAST_MK[id] != 0) && (mk_in(mk, &["final_start", "final_air_start", "final_dash", "final_dash_loop", "final_dash_end",
            "final_air_dash_end", "final_end", "final_air_end", "visual_scene_01", "visual_scene_02", "visual_scene_03", "visual_scene_04",
            "visual_scene_05"]) || VANISHED[id] || HIDDEN[id]) {
            MK_LOGS += 1; dlog(&format!("entry {} motion {:#x}", id, mk));
        }
        LAST_MK[id] = mk;
    }
    let start = mk_in(mk, &["final_start", "final_air_start"]);
    let dash = mk_in(mk, &["final_dash", "final_dash_loop"]);
    let dash_end = mk_in(mk, &["final_dash_end", "final_air_dash_end"]);
    let scene = if mk == smash::hash40("visual_scene_01") { 1 } else if mk == smash::hash40("visual_scene_02") { 2 }
        else if mk == smash::hash40("visual_scene_03") { 3 } else if mk == smash::hash40("visual_scene_04") { 4 }
        else if mk == smash::hash40("visual_scene_05") { 5 } else { 0 };

    // 0. Ridley's power-up aura at the roar, recoloured dark purple
    if start && f >= 2.0 && !AURA_DONE[id] {
        AURA_DONE[id] = true;
        EffectModule::kill_kind(boma, Hash40::new("ridley_final_aura"), false, false);
        let z = Vector3f { x: 0.0, y: 0.0, z: -3.0 }; let r = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let h = EffectModule::req_follow(boma, Hash40::new("ridley_final_aura"), Hash40::new("top"), &z, &r, 1.2, true, 0, 0, 0, 0, 0, false, false) as u32;
        if h != 0 { EffectModule::set_rgb(boma, h, 0.30, 0.02, 0.55); }
    }
    if !start { AURA_DONE[id] = false; }
    // 1. roar, then vanish + rift
    if start && f >= VANISH_FRAME && !VANISHED[id] {
        VANISHED[id] = true;
        let p = *PostureModule::pos(boma);
        RIFT_AT[id] = (p.x, p.y); RIFT_T[id] = 0; RIFT_CLOSING[id] = -1;
        dlog(&format!("entry {} vanished, rift at {:.1},{:.1}", id, p.x, p.y));
    }
    if (start && VANISHED[id]) || dash {
        if RIFT_T[id] >= 0 {
            RIFT_T[id] += 1;
            let t = RIFT_T[id] as f32;
            let open = (t / RIFT_OPEN as f32).min(1.0);
            RIFT_K[id] = 0.05 + 0.95 * (1.0 - (1.0 - open) * (1.0 - open)) + (t - RIFT_OPEN as f32).max(0.0) * 0.006;
        }
        silence(boma);
    } else if dash_end {
        // missed: the rift closes and Giratina comes back
        if RIFT_T[id] >= 0 {
            if RIFT_CLOSING[id] < 0 { RIFT_CLOSING[id] = 0; }
            RIFT_CLOSING[id] += 1;
            RIFT_K[id] = (RIFT_K[id] * 0.75).max(0.0);
            if RIFT_CLOSING[id] > 8 { rift_off(boma, id); }
        }
        VANISHED[id] = false;
        silence(boma);
    } else if scene > 0 {
        rift_off(boma, id);
        VANISHED[id] = false;
        VisibilityModule::set_whole(boma, false);
        HIDDEN[id] = true;
        CINE = true;
        silence(boma);
        if scene == 1 { CRY_DONE[id] = false; SLASH_DONE[id] = 0; }
        if scene == 4 && !CRY_DONE[id] && f >= CRY_FRAME {
            CRY_DONE[id] = true;
            SoundModule::play_se(boma, Hash40::new("vc_ridley_final01"), true, false, false, false, smash::app::enSEType(0));
        }
        if scene == 5 {
            while (SLASH_DONE[id] as usize) < SLASH_FRAMES.len() && f >= SLASH_FRAMES[SLASH_DONE[id] as usize] {
                SLASH_DONE[id] += 1;
                SoundModule::play_se(boma, Hash40::new("se_common_swing_09"), true, false, false, false, smash::app::enSEType(0));
                SoundModule::play_se(boma, Hash40::new("se_common_cutup_l"), true, false, false, false, smash::app::enSEType(0));
            }
            WHITE = (f / 8.0).clamp(0.05, 1.0); WHITE_FROM = id as i32;
        } else if WHITE_FROM == id as i32 { WHITE = 0.0; }
    } else {
        // anything else (final_end, normal play): everything back to normal
        if VANISHED[id] || RIFT_T[id] >= 0 || RIFT_K[id] > 0.0 { VANISHED[id] = false; rift_off(boma, id); }
        if HIDDEN[id] { VisibilityModule::set_whole(boma, true); HIDDEN[id] = false; CINE = false; }
        if WHITE_FROM == id as i32 { WHITE = 0.0; WHITE_FROM = -1; }
    }
}

// every fighter, every frame: the opponents turn themselves white during the black scene (no cross-object lookups)
unsafe extern "C" fn any_fighter_line(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if boma.is_null() { return; }
    if giratina_id(boma).is_some() { return; }
    let e = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    if e < 0 || e >= 8 { return; }
    let e = e as usize;
    // the cutscene victim normally lies pinned ("damage_fly_hi" -> "down_wait_u"); on the rock it stands
    if CINE && StatusModule::status_kind(boma) == FINAL_VICTIM_STATUS {
        let mk = MotionModule::motion_kind(boma);
        if mk != smash::hash40("wait") {
            MotionModule::change_motion(boma, Hash40::new("wait"), 0.0, 1.0, false, 0.0, false, false);
        }
    }
    if WHITE > 0.0 && WHITE_FROM != e as i32 {
        let c = WHITE;
        let col = smash::phx::Vector4f { x: c, y: c, z: c, w: 1.0 };
        ColorBlendModule::set_main_color(boma, &col, &col, 1.0, 1.0, 0, true);
        TINTED[e] = true;
    } else if TINTED[e] {
        ColorBlendModule::cancel_main_color(boma, 0);
        TINTED[e] = false;
    }
}

// the gunship (cutscene set) of a Giratina costume: no gunship effects or sounds; instead a dark purple aura on the
// cutscene Giratina (joints G_* of the set)
unsafe extern "C" fn gunship_quiet(_agent: &mut smash::lua2cpp::L2CAgentBase) {}

const AURA_JOINTS: [&str; 5] = ["g_hip", "g_bust", "g_head", "g_tail2", "g_tail6"];
unsafe fn aura_on(b: *mut BattleObjectModuleAccessor, size: f32) {
    let z = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    for j in AURA_JOINTS.iter() {
        let h = EffectModule::req_follow(b, Hash40::new("ridley_final_aura"), Hash40::new(j), &z, &z, size, true, 0, 0, 0, 0, 0, false, false) as u32;
        if h != 0 { EffectModule::set_rgb(b, h, 0.30, 0.02, 0.55); }
    }
}
unsafe fn aura_off(b: *mut BattleObjectModuleAccessor) {
    EffectModule::kill_kind(b, Hash40::new("ridley_final_aura"), false, true);
}
// visual_scene_02 covers cutscene scenes 2 and 3 (83 + 54 frames)
unsafe extern "C" fn gunship_eff_02(agent: &mut smash::lua2cpp::L2CAgentBase) {
    let l = agent.lua_state_agent; let b = agent.module_accessor;
    smash::app::sv_animcmd::frame(l, 1.0);
    if smash::app::sv_animcmd::is_excute(l) { aura_on(b, 2.6); }
    smash::app::sv_animcmd::frame(l, 75.0);
    if smash::app::sv_animcmd::is_excute(l) { aura_off(b); }
    smash::app::sv_animcmd::frame(l, 83.0 + 41.0);
    if smash::app::sv_animcmd::is_excute(l) { aura_on(b, 2.4); }
}
unsafe extern "C" fn gunship_eff_04(agent: &mut smash::lua2cpp::L2CAgentBase) {
    let l = agent.lua_state_agent; let b = agent.module_accessor;
    smash::app::sv_animcmd::frame(l, 1.0);
    if smash::app::sv_animcmd::is_excute(l) { aura_off(b); aura_on(b, 2.4); }
}
unsafe extern "C" fn gunship_eff_05(agent: &mut smash::lua2cpp::L2CAgentBase) {
    let l = agent.lua_state_agent; let b = agent.module_accessor;
    smash::app::sv_animcmd::frame(l, 1.0);
    if smash::app::sv_animcmd::is_excute(l) { EffectModule::kill_all(b, *EFFECT_SUB_ATTRIBUTE_NONE as u32, true, false); }
}

type InstallAcmdCostume = unsafe extern "C" fn(agent: u64, costume: CostumeArg, script: u64, category: i32, priority: i32,
                                               function: unsafe extern "C" fn(&mut smash::lua2cpp::L2CAgentBase));
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CostumeArg { data: *const usize, len: usize }
static GIR_COSTUMES: [usize; 2] = [4, 5];

unsafe fn install_gunship_scripts() {
    let mut addr: usize = 0;
    let rc = skyline::nn::ro::LookupSymbol(&mut addr, "smashline_install_acmd_script_costume\0".as_ptr() as _);
    if rc != 0 || addr == 0 { dlog("smashline_install_acmd_script_costume not found - gunship effects stay"); return; }
    let install: InstallAcmdCostume = core::mem::transmute(addr);
    let costume = CostumeArg { data: GIR_COSTUMES.as_ptr(), len: GIR_COSTUMES.len() };
    let agent = smash::hash40("ridley_gunship");
    for m in ["visualscene01", "visualscene02", "visualscene04", "visualscene05", "finalfall"].iter() {
        let eff: unsafe extern "C" fn(&mut smash::lua2cpp::L2CAgentBase) = match *m {
            "visualscene02" => gunship_eff_02, "visualscene04" => gunship_eff_04, "visualscene05" => gunship_eff_05, _ => gunship_quiet };
        install(agent, costume, smash::hash40(&format!("effect_{}", m)), 1, 2, eff);
        install(agent, costume, smash::hash40(&format!("sound_{}", m)), 2, 2, gunship_quiet);
    }
    dlog("gunship effect/sound scripts emptied for costumes 8/9");
}

// smashline's plain (non-costume) line callback API, looked up by name so it works with any smashline 2 build
type InstallLine = unsafe extern "C" fn(agent: Option<core::num::NonZeroU64>, line: i32, function: *const ());
const STATUS_LINE_MAIN: i32 = 1;            // smashline StatusLine::Main: once per frame, form logic
const STATUS_LINE_MAP_CORRECTION: i32 = 8;  // later in the frame: re-apply meshes/joints after the animation
const STATUS_LINE_FIX_CAMERA: i32 = 9;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
static COMMON_NRO: AtomicBool = AtomicBool::new(false);
static RIDLEY_NRO: AtomicBool = AtomicBool::new(false);
static INSTALLED: AtomicBool = AtomicBool::new(false);
static SECS: AtomicU32 = AtomicU32::new(0);

// Registering with smashline during boot crashes the game (v5-v7); registering after the boot works (v8).
// smashline copies the callback list into each fighter when that fighter is created, so v9 registers the
// moment Ridley's fighter module loads for the first match, before any Ridley exists.
fn nro_loaded(info: &skyline::nro::NroInfo) {
    if info.name == "common" {
        COMMON_NRO.store(true, Ordering::Relaxed);
        dlog(&format!("t+{}s nro loaded: common", SECS.load(Ordering::Relaxed)));
    }
    if info.name == "ridley" {
        RIDLEY_NRO.store(true, Ordering::Relaxed);
        dlog(&format!("t+{}s nro loaded: ridley", SECS.load(Ordering::Relaxed)));
        if COMMON_NRO.load(Ordering::Relaxed) { unsafe { install_now("ridley module load"); } }
    }
}

unsafe fn install_now(why: &str) {
    if INSTALLED.swap(true, Ordering::SeqCst) { return; }
    let mut addr: usize = 0;
    let rc = skyline::nn::ro::LookupSymbol(&mut addr, "smashline_install_line_callback\0".as_ptr() as _);
    if rc != 0 || addr == 0 {
        dlog(&format!("smashline_install_line_callback not found (rc {:#x}) - forms disabled", rc));
        return;
    }
    let install: InstallLine = core::mem::transmute(addr);
    let agent = core::num::NonZeroU64::new(smash::hash40("ridley"));
    install(agent, STATUS_LINE_MAIN, giratina_line as *const ());
    install(agent, STATUS_LINE_MAP_CORRECTION, giratina_late as *const ());
    install(agent, STATUS_LINE_FIX_CAMERA, giratina_late as *const ());
    install(core::num::NonZeroU64::new(smash::hash40("fighter")), STATUS_LINE_MAIN, any_fighter_line as *const ());
    install_gunship_scripts();
    dlog(&format!("t+{}s callbacks installed ({})", SECS.load(Ordering::Relaxed), why));
}

#[skyline::main(name = "giratina_skin")]
pub fn main() {
    dlog("giratina_forms v13 loaded");
    let _ = skyline::nro::add_hook(nro_loaded);
    std::thread::spawn(|| {
        let tick = std::time::Duration::from_secs(1);
        for s in 1..=900u32 {
            std::thread::sleep(tick);
            SECS.store(s, Ordering::Relaxed);
            // fallback in case the module-load path was missed
            if RIDLEY_NRO.load(Ordering::Relaxed) && !INSTALLED.load(Ordering::Relaxed) { unsafe { install_now("fallback"); } }
        }
    });
}

static mut LINE_CALLS: u32 = 0;
unsafe extern "C" fn giratina_line(fighter: &mut L2CFighterCommon) {
    LINE_CALLS += 1;
    if LINE_CALLS <= 2 { dlog(&format!("main line call {}", LINE_CALLS)); }
    let p = fighter as *mut L2CFighterCommon;
    if p.is_null() { return; }
    giratina_frame(p);
}

static mut LATE_CALLS: u32 = 0;
unsafe extern "C" fn giratina_late(fighter: &mut L2CFighterCommon) {
    LATE_CALLS += 1;
    if LATE_CALLS <= 2 { dlog(&format!("late line call {}", LATE_CALLS)); }
    let p = fighter as *mut L2CFighterCommon;
    if p.is_null() { return; }
    let boma = (&*p).module_accessor;
    if let Some(id) = giratina_id(boma) {
        apply_form(boma, id);
        if VANISHED[id] || HIDDEN[id] { silence(boma); }
    }
}
