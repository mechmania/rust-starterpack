#![allow(dead_code)]

use anyhow::Context;
use memmap::MmapMut;
use pastey::paste;
use std::{
    fs::OpenOptions,
    hint,
    mem::offset_of,
    path::Path,
    sync::atomic::{AtomicU8, Ordering},
    time::Duration,
};
use std::sync::OnceLock;
use crate::core::{
    util::Vec2,
    config::{ GameConfig, NUM_PLAYERS },
    state::{ Team, GameState, PlayerAction, TeamPair },
};

#[repr(u8)]
pub enum EngineStatus {
    Ready = 0,
    Busy = 1,
}

macro_rules! define_protocols {
    (
        $(
            $name:ident: ($msg:ty, $resp:ty)
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum ProtocolId {
            $(
                $name,
            )*
        }
        paste! {
            #[derive(Clone)]
            #[repr(u8, C)]
            pub enum ProtocolUnion {
                $(
                    [<$name Msg>]($msg),
                    [<$name Response>]($resp),
                )*
            }
        }
    };
}

#[derive(Clone)]
#[repr(C)]
pub struct HandshakeMsg{
    pub team: u8,
    pub config: GameConfig
}

type Score = TeamPair<u32>;

pub const HANDSHAKE_BOT: u64 = 0xabe119c019aaffcc;

define_protocols! {
    Handshake: (HandshakeMsg, u64),
    Reset: (Score, [Vec2; NUM_PLAYERS as usize]),
    Tick: (GameState, [PlayerAction; NUM_PLAYERS as usize])
}

#[repr(C)]
struct Shm {
    sync: AtomicU8,
    protocol: ProtocolUnion,
}

#[inline(never)]
async fn poll(au8: &AtomicU8, cmp: u8) {
    for i in 0.. {
        if au8.load(Ordering::Acquire) == cmp {
            return;
        }
        match i {
            0..100 => hint::spin_loop(),
            100..1000 => std::thread::yield_now(),
            _ => tokio::time::sleep(Duration::from_micros(i / 10)).await,
        }

    }
}

pub struct Strategy {
    pub on_reset: Box<dyn Fn(&TeamPair<u32>) -> [Vec2; NUM_PLAYERS as usize]>,
    pub on_tick: Box<dyn Fn(&GameState) -> [PlayerAction; NUM_PLAYERS as usize]>,
}

// safe because we only grab one byte
#[inline]
fn deref_sync<'a>(mmap: &'a [u8]) -> &'a AtomicU8 {
    unsafe { &*(mmap.as_ptr().add(offset_of!(Shm, sync)) as *const AtomicU8) }
}

static TEAM: OnceLock<u8> = OnceLock::new();
static CONFIG: OnceLock<GameConfig> = OnceLock::new();

pub fn get_real_team() -> u8 {
    *TEAM.get().unwrap()
}

pub fn get_config() -> &'static GameConfig {
    CONFIG.get().unwrap()
}

pub struct EngineChannel {
    mmap: MmapMut,
}

impl EngineChannel {
    pub fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .with_context(|| "unable to open backing file for engine channel")?;

        Ok(Self {
            mmap: unsafe {
                MmapMut::map_mut(&file).with_context(|| "unable to memory map backing file")?
            },
        })
    }

    pub async fn handle_handshake(&self) -> anyhow::Result<()> {
        let sync = deref_sync(&self.mmap);
        poll(
            sync, 
            EngineStatus::Ready as u8
        ).await;

        // safe to deref because engine is trusted
        let msg = unsafe { &mut* (self.mmap.as_ptr().add(offset_of!(Shm, protocol)) as *mut ProtocolUnion) };

        let ProtocolUnion::HandshakeMsg(
            HandshakeMsg { 
                team, 
                config 
            }
        ) = msg else {
            anyhow::bail!("did not recieve handshake message")
        };

        TEAM.set(*team).unwrap();
        CONFIG.set(config.clone()).unwrap();

        *msg = ProtocolUnion::HandshakeResponse(HANDSHAKE_BOT);

        sync.store(EngineStatus::Busy as u8, Ordering::Release);
        Ok(())
    }

    pub async fn handle_msg(&self, strategy: &Strategy) {
        let sync = deref_sync(&self.mmap);
        poll(
            sync, 
            EngineStatus::Ready as u8
        ).await;

        // safe to deref because engine is trusted
        let msg = unsafe { &mut* (self.mmap.as_ptr().add(offset_of!(Shm, protocol)) as *mut ProtocolUnion) };

        let response = match msg {
            ProtocolUnion::ResetMsg(score) => ProtocolUnion::ResetResponse((strategy.on_reset)(score)),
            ProtocolUnion::TickMsg(state) => ProtocolUnion::TickResponse((strategy.on_tick)(state)),
            _ => panic!()
        };

        *msg = response;

        sync.store(EngineStatus::Busy as u8, Ordering::Release);
    }
}

