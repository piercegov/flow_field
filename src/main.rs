#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{prelude::*, window::{WindowResized, WindowResolution}, core_pipeline::{tonemapping::Tonemapping, bloom::BloomSettings}};
use bevy_hanabi::prelude::*;
use libnoise::prelude::*;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

const NUM_PARTICLES: f32 = 2000.;
const PARTICLE_RADIUS: f32 = 2.;

const SPEED_FACTOR: f32 = 1.;

const LUMINOSITY: f32 = 20.;

fn main() {
    // seed based on time
    let seed: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
               resolution: WindowResolution::new(WIDTH as f32, HEIGHT as f32),
               ..default()
            }),
            ..Default::default()
        }))
        .add_plugins(HanabiPlugin)
        .insert_resource(WindowSize {
            width: WIDTH as f32,
            height: HEIGHT as f32
        })
        .insert_resource(NoiseGen { gen: Source::perlin(seed), seed, noise_scale: 0.005 })
        .add_systems(Startup, (setup_camera, add_particles))
        .add_systems(PostStartup, draw_trails)
        .add_systems(Update, (move_particles, keyboard_input)) // check_field, 
        .add_systems(Update, window_resize)
        .run();
}


#[derive(Component, Copy, Clone)]
struct Particle {
    x: f32,
    y: f32,
}

impl Particle {
    fn random(width: f32, height: f32) -> Self {
        let x = rand::random::<f32>() * width as f32 - (width as f32 / 2.);
        let y = rand::random::<f32>() * height as f32 - (height as f32 / 2.);

        Particle {
            x,
            y,
        }
    }

}

#[derive(Resource)]
struct NoiseGen {
    gen: Perlin<2>,
    noise_scale: f32,
    seed: u64,
}

impl NoiseGen {
    fn gen(&self, x: f32, y: f32, width: f32, height: f32) -> f64 {
        // correct for origin being in the center, we want to have bottom left be the origin
        let x = x + (width as f32 / 2.);
        let y = y + (height as f32 / 2.);
        self.gen.sample([x as f64 * self.noise_scale as f64, y as f64 * self.noise_scale as f64])
    }
}


fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        tonemapping: Tonemapping::None,
        ..default()
    }, BloomSettings::default()));
}

#[derive(Resource)]
struct WindowSize {
    width: f32,
    height: f32
}

fn window_resize(mut resize_reader: EventReader<WindowResized>, 
                 mut window_size: ResMut<WindowSize>) {
    for e in resize_reader.iter() {
        window_size.width = e.width;
        window_size.height = e.height;
    }
}

fn add_particles(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, window_size: Res<WindowSize>) {
    for _ in 0..NUM_PARTICLES as usize {
            // add a particle
            let particle: Particle = Particle::random(window_size.width, window_size.height);

            commands.spawn(particle);
    }
}

fn move_particles(mut particles: Query<(&mut Particle, &mut Transform)>, window_size: Res<WindowSize>, noise_gen: Res<NoiseGen>) {
    for (mut particle, mut transform) in particles.iter_mut() {
        // check if the particle is out of bounds
        if (particle.x >= (window_size.width / 2f32) as f32 || particle.y >= (window_size.height / 2f32) as f32 || particle.x <= -(window_size.width / 2f32) as f32 || particle.y <= -(window_size.height / 2f32) as f32) {
            // move the particle to a random position on the screen
            particle.x = -window_size.width / 2. + rand::random::<f32>() * window_size.width;
            particle.y = -window_size.height / 2. + rand::random::<f32>() * window_size.height;
        }

        let sample = noise_gen.gen(particle.x, particle.y, window_size.width, window_size.height) * 2 as f64 * std::f64::consts::PI;
        
        particle.x += (sample.cos()) as f32 * SPEED_FACTOR;
        particle.y += (sample.sin()) as f32 * SPEED_FACTOR;

        transform.translation.x = particle.x;
        transform.translation.y = particle.y;
    }
}

fn keyboard_input(keys: Res<Input<KeyCode>>, mut noise_gen: ResMut<NoiseGen>) {
    if keys.just_pressed(KeyCode::Space) {
        noise_gen.seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64;
        noise_gen.gen = Source::perlin(noise_gen.seed);
    }

    if keys.just_pressed(KeyCode::Up) {
        noise_gen.noise_scale /= 2.;
    } else if keys.just_pressed(KeyCode::Down) {
        noise_gen.noise_scale *= 2.;
    }
}

fn draw_trails(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>, particles: Query<Entity, With<Particle>>) {
    for entity in particles.iter() {

        let writer = ExprWriter::new();
        let age = writer.lit(0.0).expr();
        let init_age = SetAttributeModifier::new(Attribute::AGE, age);

        let lifetime = writer.lit(1.).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
        
        let mut gradient = Gradient::new();
        gradient.add_key(0.0, Vec4::new(LUMINOSITY, LUMINOSITY, LUMINOSITY, 0.2)); // Start color: white, fully opaque
        gradient.add_key(0.5, Vec4::new(LUMINOSITY, LUMINOSITY, LUMINOSITY, 0.05));
        gradient.add_key(1., Vec4::new(LUMINOSITY, LUMINOSITY, LUMINOSITY, 0.0)); // End color: white, fully transparent

        let init_pos = 
            SetPositionCircleModifier {
                center: writer.lit(Vec3::new(0., 0., 0.)).expr(),
                radius: writer.lit(0.001).expr(),
                axis: writer.lit(Vec3::Z).expr(),
                dimension: ShapeDimension::Surface
            };

        let init_vel = 
            SetVelocityCircleModifier {
                center: writer.lit(Vec3::new(0.0, 0., 0.)).expr(),
                axis: writer.lit(Vec3::Z).expr(),
                speed: writer.lit(0.0001).expr()
            };
        
        let effect = effects.add(
            EffectAsset::new(1024, Spawner::rate(60.0.into()), writer.finish())
            .with_name("trail")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec2::splat(PARTICLE_RADIUS)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier { gradient })
        );
    
        commands.entity(entity).insert(ParticleEffectBundle {
            effect: ParticleEffect::new(effect).with_z_layer_2d(Some(0.)),
            ..default()
        });
    }
}
