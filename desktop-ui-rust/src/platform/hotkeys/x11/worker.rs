use std::collections::{BTreeSet, HashMap};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use iced::futures::channel::mpsc as fmpsc;
use x11rb::connection::Connection;
use x11rb::protocol::xkb;
use x11rb::protocol::xproto::{self, ConnectionExt as _, KeyButMask, Keycode, ModMask};
use x11rb::rust_connection::RustConnection;

use crate::settings::{HotkeyAction, HotkeyBinding, PhysicalKeyId};

use super::notice::GlobalHotkeyNotice;
use super::state::GlobalHotkeysState;

const REPEAT_DELAY: Duration = Duration::from_millis(500);
const REPEAT_INTERVAL: Duration = Duration::from_millis(250);
const POLL_SLEEP: Duration = Duration::from_millis(10);

enum Command {
    Apply {
        enabled: bool,
        bindings: Vec<(HotkeyAction, HotkeyBinding)>,
    },
    Stop,
}

#[derive(Clone)]
pub struct GlobalHotkeysHandle {
    commands: mpsc::Sender<Command>,
}

impl GlobalHotkeysHandle {
    pub fn apply(&self, enabled: bool, bindings: &std::collections::BTreeMap<HotkeyAction, HotkeyBinding>) {
        let bindings = bindings.iter().map(|(a, b)| (*a, *b)).collect();
        let _ = self.commands.send(Command::Apply { enabled, bindings });
    }

    pub fn stop(&self) {
        let _ = self.commands.send(Command::Stop);
    }
}

pub fn spawn() -> (GlobalHotkeysHandle, fmpsc::UnboundedReceiver<GlobalHotkeyNotice>) {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (notice_tx, notice_rx) = fmpsc::unbounded();

    let _ = thread::Builder::new()
        .name("x11-global-hotkeys".to_owned())
        .spawn(move || run(cmd_rx, notice_tx));

    (GlobalHotkeysHandle { commands: cmd_tx }, notice_rx)
}

fn run(commands: Receiver<Command>, notices: fmpsc::UnboundedSender<GlobalHotkeyNotice>) {
    let conn = match RustConnection::connect(None) {
        Ok((conn, screen_num)) => (conn, screen_num),
        Err(error) => {
            send_status(
                &notices,
                GlobalHotkeysState::Unavailable(format!(
                    "не удалось подключиться к X11: {error}"
                )),
            );
            drain_commands(commands, &notices);
            return;
        }
    };

    let (conn, screen_num) = conn;
    let root = match conn.setup().roots.get(screen_num) {
        Some(screen) => screen.root,
        None => {
            send_status(
                &notices,
                GlobalHotkeysState::Unavailable("X11: экран не найден".to_owned()),
            );
            drain_commands(commands, &notices);
            return;
        }
    };

    let keyboard = match load_keyboard(&conn) {
        Ok(value) => value,
        Err(reason) => {
            send_status(&notices, GlobalHotkeysState::Unavailable(reason));
            drain_commands(commands, &notices);
            return;
        }
    };

    let _ = enable_detectable_auto_repeat(&conn);

    let mut state = WorkerState {
        conn,
        root,
        keyboard,
        notices,
        grabbed: HashMap::new(),
        held: HashMap::new(),
        repeating: HashMap::new(),
    };

    loop {
        match commands.try_recv() {
            Ok(Command::Apply { enabled, bindings }) => state.apply(enabled, &bindings),
            Ok(Command::Stop) => {
                state.clear_grabs();
                let _ = state.conn.flush();
                send_status(&state.notices, GlobalHotkeysState::Off);
                return;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                state.clear_grabs();
                let _ = state.conn.flush();
                return;
            }
        }

        match state.conn.poll_for_event() {
            Ok(Some(event)) => state.handle_event(event),
            Ok(None) => {}
            Err(_) => {
                state.clear_grabs();
                send_status(
                    &state.notices,
                    GlobalHotkeysState::Unavailable(
                        "X11: подключение к серверу оборвалось".to_owned(),
                    ),
                );
                return;
            }
        }

        state.tick_repeats();
        thread::sleep(POLL_SLEEP);
    }
}

fn drain_commands(commands: Receiver<Command>, notices: &fmpsc::UnboundedSender<GlobalHotkeyNotice>) {
    while let Ok(command) = commands.recv() {
        match command {
            Command::Stop => {
                send_status(notices, GlobalHotkeysState::Off);
                return;
            }
            Command::Apply { enabled, .. } if !enabled => {
                send_status(notices, GlobalHotkeysState::Off);
            }
            Command::Apply { .. } => {}
        }
    }
}

struct KeyboardInfo {
    min_keycode: Keycode,
    keysyms_per_keycode: u8,
    keysyms: Vec<xproto::Keysym>,
    num_lock_mask: u16,
    alt_mask: Option<u16>,
}

fn load_keyboard(conn: &RustConnection) -> Result<KeyboardInfo, String> {
    let setup = conn.setup();
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;
    if max_keycode < min_keycode {
        return Err("X11: неверный диапазон клавиш".to_owned());
    }
    let count = max_keycode - min_keycode + 1;
    let mapping = conn
        .get_keyboard_mapping(min_keycode, count)
        .map_err(|e| format!("X11: не удалось получить раскладку: {e}"))?
        .reply()
        .map_err(|e| format!("X11: раскладка клавиатуры недоступна: {e}"))?;

    let keysyms_per_keycode = mapping.keysyms_per_keycode;
    let keysyms = mapping.keysyms;

    let modifiers = conn
        .get_modifier_mapping()
        .map_err(|e| format!("X11: не удалось получить модификаторы: {e}"))?
        .reply()
        .map_err(|e| format!("X11: модификаторы недоступны: {e}"))?;

    let keycodes_per_modifier = modifiers.keycodes_per_modifier() as usize;
    let mod_keycodes = modifiers.keycodes;
    let num_lock_mask = modifier_mask_for_keysym(
        &[keysym_num_lock()],
        min_keycode,
        keysyms_per_keycode,
        &keysyms,
        &mod_keycodes,
        keycodes_per_modifier,
    )
    .unwrap_or(0);
    let alt_mask = modifier_mask_for_keysym(
        &[keysym_alt_l(), keysym_alt_r()],
        min_keycode,
        keysyms_per_keycode,
        &keysyms,
        &mod_keycodes,
        keycodes_per_modifier,
    );

    Ok(KeyboardInfo {
        min_keycode,
        keysyms_per_keycode,
        keysyms,
        num_lock_mask,
        alt_mask,
    })
}

fn keysym_num_lock() -> xproto::Keysym {
    0xff7f
}
fn keysym_alt_l() -> xproto::Keysym {
    0xffe9
}
fn keysym_alt_r() -> xproto::Keysym {
    0xffea
}

fn keysyms_for_keycode(
    keycode: Keycode,
    min_keycode: Keycode,
    keysyms_per_keycode: u8,
    keysyms: &[xproto::Keysym],
) -> &[xproto::Keysym] {
    if keycode < min_keycode || keysyms_per_keycode == 0 {
        return &[];
    }
    let index = (keycode - min_keycode) as usize * keysyms_per_keycode as usize;
    let end = index.saturating_add(keysyms_per_keycode as usize).min(keysyms.len());
    if index >= end {
        &[]
    } else {
        &keysyms[index..end]
    }
}

fn keycode_for_keysym(
    wanted: xproto::Keysym,
    min_keycode: Keycode,
    keysyms_per_keycode: u8,
    keysyms: &[xproto::Keysym],
) -> Option<Keycode> {
    if keysyms_per_keycode == 0 {
        return None;
    }
    let per = keysyms_per_keycode as usize;
    for (i, chunk) in keysyms.chunks(per).enumerate() {
        if chunk.iter().any(|k| *k == wanted) {
            return Some(min_keycode.saturating_add(i as Keycode));
        }
    }
    None
}

fn modifier_mask_for_keysym(
    wanted: &[xproto::Keysym],
    min_keycode: Keycode,
    keysyms_per_keycode: u8,
    keysyms: &[xproto::Keysym],
    mod_keycodes: &[Keycode],
    keycodes_per_modifier: usize,
) -> Option<u16> {
    if keycodes_per_modifier == 0 {
        return None;
    }
    for row in 3usize..8usize {
        let start = row * keycodes_per_modifier;
        let end = start + keycodes_per_modifier;
        if end > mod_keycodes.len() {
            continue;
        }
        let found = mod_keycodes[start..end].iter().any(|kc| {
            *kc != 0
                && keysyms_for_keycode(*kc, min_keycode, keysyms_per_keycode, keysyms)
                    .iter()
                    .any(|k| wanted.contains(k))
        });
        if found {
            return Some(1u16 << row);
        }
    }
    None
}

fn enable_detectable_auto_repeat(conn: &RustConnection) -> Result<(), ()> {
    let ext = match xkb::use_extension(conn, 1, 0) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(_) => return Err(()),
        },
        Err(_) => return Err(()),
    };
    if !ext.supported {
        return Err(());
    }
    let flag = xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT;
    let reply = match xkb::per_client_flags(
        conn,
        256u16,
        flag,
        flag,
        xkb::BoolCtrl::from(0u32),
        xkb::BoolCtrl::from(0u32),
        xkb::BoolCtrl::from(0u32),
    ) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(_) => return Err(()),
        },
        Err(_) => return Err(()),
    };
    if reply.value.contains(flag) {
        Ok(())
    } else {
        Err(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GrabKey {
    keycode: Keycode,
    modifiers: u16,
}

struct RepeatState {
    next_at: Instant,
    started: bool,
}

struct WorkerState {
    conn: RustConnection,
    root: xproto::Window,
    keyboard: KeyboardInfo,
    notices: fmpsc::UnboundedSender<GlobalHotkeyNotice>,
    grabbed: HashMap<GrabKey, HotkeyAction>,
    held: HashMap<Keycode, HotkeyAction>,
    repeating: HashMap<Keycode, RepeatState>,
}

impl WorkerState {
    fn apply(&mut self, enabled: bool, bindings: &[(HotkeyAction, HotkeyBinding)]) {
        self.clear_grabs();
        if !enabled {
            send_status(&self.notices, GlobalHotkeysState::Off);
            let _ = self.conn.flush();
            return;
        }
        let (grabbed, rejected) = self.grab_all(bindings);
        self.grabbed = grabbed;
        send_status(
            &self.notices,
            GlobalHotkeysState::Active { rejected },
        );
        let _ = self.conn.flush();
    }

    fn grab_all(
        &self,
        bindings: &[(HotkeyAction, HotkeyBinding)],
    ) -> (HashMap<GrabKey, HotkeyAction>, BTreeSet<HotkeyAction>) {
        let mut taken = HashMap::new();
        let mut rejected = BTreeSet::new();
        let lock = u16::from(ModMask::LOCK);
        let num_lock = self.keyboard.num_lock_mask;
        let lock_variants = {
            let mut v = vec![0u16, lock, num_lock, lock | num_lock];
            v.sort_unstable();
            v.dedup();
            v
        };

        for (action, binding) in bindings {
            let modifiers = match self.binding_modifiers(*binding) {
                Some(m) => m,
                None => {
                    rejected.insert(*action);
                    continue;
                }
            };
            let keycode = match self.binding_keycode(binding.key) {
                Some(k) => k,
                None => {
                    rejected.insert(*action);
                    continue;
                }
            };

            let masks: Vec<u16> = lock_variants.iter().map(|extra| modifiers | *extra).collect();
            let ok = self.grab_key_variants(keycode, &masks);
            if ok {
                for mask in masks {
                    taken.insert(
                        GrabKey {
                            keycode,
                            modifiers: mask & relevant_mask(self.keyboard.alt_mask),
                        },
                        *action,
                    );
                }
            } else {
                rejected.insert(*action);
            }
        }
        (taken, rejected)
    }

    fn binding_modifiers(&self, binding: HotkeyBinding) -> Option<u16> {
        let mut mask = 0u16;
        if binding.shift {
            mask |= u16::from(ModMask::SHIFT);
        }
        if binding.ctrl {
            mask |= u16::from(ModMask::CONTROL);
        }
        if binding.alt {
            mask |= self.keyboard.alt_mask?;
        }
        Some(mask)
    }

    fn binding_keycode(&self, key: PhysicalKeyId) -> Option<Keycode> {
        match key {
            PhysicalKeyId::Code(_code) => {
                let keysym = crate::settings::hotkey::x11::keysym(key)?;
                keycode_for_keysym(
                    keysym,
                    self.keyboard.min_keycode,
                    self.keyboard.keysyms_per_keycode,
                    &self.keyboard.keysyms,
                )
            }
            PhysicalKeyId::Xkb(value) => {
                let setup = self.conn.setup();
                let min = setup.min_keycode as u32;
                let max = setup.max_keycode as u32;
                if value >= min && value <= max && value <= u8::MAX as u32 {
                    Some(value as Keycode)
                } else {
                    None
                }
            }
        }
    }

    fn grab_key_variants(&self, keycode: Keycode, masks: &[u16]) -> bool {
        let mut granted = Vec::with_capacity(masks.len());
        for mask in masks {
            let cookie = self.conn.grab_key(
                false,
                self.root,
                ModMask::from(*mask),
                keycode,
                xproto::GrabMode::ASYNC,
                xproto::GrabMode::ASYNC,
            );
            let ok = match cookie {
                Ok(cookie) => cookie.check().is_ok(),
                Err(_) => false,
            };
            granted.push(ok);
        }
        if granted.iter().all(|v| *v) {
            return true;
        }
        for (mask, ok) in masks.iter().zip(granted.iter()) {
            if *ok {
                let _ = self.conn.ungrab_key(keycode, self.root, ModMask::from(*mask));
            }
        }
        false
    }

    fn clear_grabs(&mut self) {
        self.grabbed.clear();
        self.held.clear();
        self.repeating.clear();
        let _ = self.conn.ungrab_key(0u8, self.root, ModMask::ANY);
    }

    fn handle_event(&mut self, event: x11rb::protocol::Event) {
        match event {
            x11rb::protocol::Event::KeyPress(ev) => self.on_press(ev),
            x11rb::protocol::Event::KeyRelease(ev) => self.on_release(ev),
            _ => {}
        }
    }

    fn on_press(&mut self, ev: xproto::KeyPressEvent) {
        let relevant = relevant_mask(self.keyboard.alt_mask);
        let state = keybut_state_bits(ev.state) & relevant;
        let action = match self.grabbed.get(&GrabKey {
            keycode: ev.detail,
            modifiers: state,
        }) {
            Some(action) => *action,
            None => return,
        };

        if self.held.insert(ev.detail, action).is_some() {
            return;
        }

        let _ = self.notices.unbounded_send(GlobalHotkeyNotice::Action(action));
        if action.repeats() {
            self.repeating.insert(
                ev.detail,
                RepeatState {
                    next_at: Instant::now() + REPEAT_DELAY,
                    started: false,
                },
            );
        }
    }

    fn on_release(&mut self, ev: xproto::KeyReleaseEvent) {
        self.held.remove(&ev.detail);
        self.repeating.remove(&ev.detail);
    }

    fn tick_repeats(&mut self) {
        let now = Instant::now();
        let due: Vec<(Keycode, HotkeyAction)> = self
            .repeating
            .iter()
            .filter_map(|(keycode, state)| {
                if now >= state.next_at {
                    self.held.get(keycode).copied().map(|action| (*keycode, action))
                } else {
                    None
                }
            })
            .collect();

        for (keycode, action) in due {
            let _ = self.notices.unbounded_send(GlobalHotkeyNotice::Action(action));
            if let Some(state) = self.repeating.get_mut(&keycode) {
                state.started = true;
                state.next_at = now + REPEAT_INTERVAL;
            }
        }
    }
}

fn relevant_mask(alt_mask: Option<u16>) -> u16 {
    u16::from(ModMask::SHIFT) | u16::from(ModMask::CONTROL) | alt_mask.unwrap_or(0)
}

fn keybut_state_bits(state: KeyButMask) -> u16 {
    state.into()
}

fn send_status(notices: &fmpsc::UnboundedSender<GlobalHotkeyNotice>, state: GlobalHotkeysState) {
    let _ = notices.unbounded_send(GlobalHotkeyNotice::Status(state));
}
