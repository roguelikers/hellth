use std::time::Duration;

use bevy::{ecs::system::Command, prelude::*};
use bevy_kira_audio::prelude::*;

use super::{actions::play_sfx, procgen::{LevelDepth, ProcGenEvent}, GameStates};

fn play_music(asset_server: Res<AssetServer>, audio: Res<Audio>, mut settings: ResMut<GameAudioSettings>) {
    let bgm = audio.play(asset_server.load("sounds/the_pit.ogg"))
        .fade_in(AudioTween::linear(Duration::from_secs_f32(2.0)))
        .with_volume(0.1).looped().handle();

    settings.music = Some(bgm);
}

#[derive(Resource)]
pub struct GameAudioSettings {
    pub music_volume: f64,
    pub sfx_volume: f64,
    pub music: Option<Handle<AudioInstance>>,
    pub boss: Option<Handle<AudioInstance>>,
}

impl Default for GameAudioSettings {
    fn default() -> Self {
        Self { music_volume: 0.2, sfx_volume: 1.0, music: None, boss: None }
    }
}

pub struct SfxCommand { 
    pub name: String
}

pub struct SfxRevCommand { 
    pub name: String
}

impl Command for SfxCommand {
    fn apply(self, world: &mut World) {
        play_sfx(&self.name, world);
    }
}

impl Command for SfxRevCommand {
    fn apply(self, world: &mut World) {
        if let Some(settings) = world.get_resource::<GameAudioSettings>() {
            if let Some(asset_server) = world.get_resource::<AssetServer>() {
                if let Some(audio) = world.get_resource::<Audio>() {
                    audio.play(asset_server.load(format!("sounds/{}.ogg", self.name))).reverse().with_volume(settings.sfx_volume);
                }
            }
        }
    }
}

fn change_music(
        asset_server: Res<AssetServer>, 
        audio: Res<Audio>, 
        mut settings: ResMut<GameAudioSettings>, 
        depth: Res<LevelDepth>, 
        mut procgen_events: EventReader<ProcGenEvent>,
        mut audio_instances: ResMut<Assets<AudioInstance>>) {

    for e in procgen_events.read() {
        if *e == ProcGenEvent::NextLevel && depth.0 == 5 {
            if let Some(music_instance) = settings.music.as_ref() {
                if let Some(music) = audio_instances.get_mut(music_instance) {
                    music.set_volume(0.0, AudioTween::linear(Duration::from_secs_f32(2.0)));
                }
            }

            if settings.boss.is_none() {
                settings.boss = Some(audio.play(asset_server.load("sounds/core.ogg"))
                    .fade_in(AudioTween::linear(Duration::from_secs_f32(2.0)))
                    .start_from(1.0)
                    .with_volume(settings.music_volume).looped().handle());
            } else {
                let _ = settings.boss.as_ref().map(|b| {
                    if let Some(music) = audio_instances.get_mut(b) {
                        music.set_volume(settings.music_volume, AudioTween::linear(Duration::from_secs_f32(2.0)));
                    }
                });
            }
        } else if *e == ProcGenEvent::RestartWorld && settings.boss.is_some() { // we leave it alone
            let _ = settings.boss.as_ref().map(|b| {
                if let Some(music) = audio_instances.get_mut(b) {
                    music.set_volume(0.0, AudioTween::linear(Duration::from_secs_f32(2.0)));
                }
            });

            if let Some(music_instance) = settings.music.as_ref() {
                if let Some(music) = audio_instances.get_mut(music_instance) {
                    music.set_volume(settings.music_volume, AudioTween::linear(Duration::from_secs_f32(2.0)));
                }
            }
        }
    }
}

fn control_audio(audio: Res<Audio>, input: Res<Input<KeyCode>>, mut audio_settings: ResMut<GameAudioSettings>, mut audio_instances: ResMut<Assets<AudioInstance>>) {
    let mut dv = 0.0;
    if input.just_pressed(KeyCode::Plus) || input.just_pressed(KeyCode::Equals) {
        dv = 1.0;
    } else if input.just_pressed(KeyCode::Minus) {
        dv = -1.0;
    }

    if dv != 0.0 {
        audio_settings.music_volume = (audio_settings.music_volume + dv * 0.05).clamp(0.0, 1.0);
        audio_settings.sfx_volume = (audio_settings.sfx_volume + dv * 0.1).clamp(0.0, 1.0);

        if let Some(music_instance) = audio_settings.music.as_ref() {
            if let Some(music) = audio_instances.get_mut(music_instance) {
                music.set_volume(audio_settings.music_volume, AudioTween::linear(Duration::from_secs_f32(0.1)));
            }
        }
    }
}

pub struct SvarogMusicPlugin;

impl Plugin for SvarogMusicPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.add_plugins(AudioPlugin)
            .init_resource::<GameAudioSettings>()
            .add_systems(Startup, play_music)
            .add_systems(Update, control_audio)
            .add_systems(Update, change_music.run_if(in_state(GameStates::Game)).run_if(on_event::<ProcGenEvent>()));
    }
}