use std::{
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        mpsc,
        Arc,
        Mutex,
    },
    thread,
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::Payload;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    CurrentUserUpdate,
    GuildStatus,
    GuildCreate,
    ChannelCreate,
    RelationshipUpdate,
    VoiceChannelSelect,
    VoiceStateCreate,
    VoiceStateDelete,
    VoiceStateUpdate,
    VoiceSettingsUpdate,
    VoiceSettingsUpdate2,
    VoiceConnectionStatus,
    SpeakingStart,
    SpeakingStop,
    GameJoin,
    GameSpectate,
    ActivityJoin,
    ActivityJoinRequest,
    ActivitySpectate,
    ActivityInvite,
    NotificationCreate,
    MessageCreate,
    MessageUpdate,
    MessageDelete,
    LobbyDelete,
    LobbyUpdate,
    LobbyMemberConnect,
    LobbyMemberDisconnect,
    LobbyMemberUpdate,
    LobbyMessage,
    CaptureShortcutChange,
    Overlay,
    OverlayUpdate,
    EntitlementCreate,
    EntitlementDelete,
    UserAchievementUpdate,
    Ready,
    Error,
}

pub struct EventHandler {
    emitter: mpsc::Sender<(Event, Payload)>,
    listener: Arc<Mutex<mpsc::Receiver<(Event, Payload)>>>,
    running: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (emitter, listener) = mpsc::channel();
        EventHandler {
            emitter,
            listener: Arc::new(Mutex::new(listener)),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn emit(&self, ev: Event, payload: Payload) {
        self.emitter.send((ev, payload)).unwrap();
    }

    pub fn listen<F>(&self, mut callback: F, ev: Event)
    where
        F: FnMut(Payload) + Send + 'static,
    {
        let listener = Arc::clone(&self.listener);
        let running = Arc::clone(&self.running);
        thread::spawn(move || {
            let listener = listener.lock().unwrap();
            while running.load(Ordering::SeqCst) {
                match listener.recv() {
                    Ok(event) => {
                        if ev == event.0 {
                            callback(event.1);
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
