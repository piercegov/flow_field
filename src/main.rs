#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{prelude::*, window::{WindowResized, WindowResolution}, core_pipeline::{tonemapping::Tonemapping, bloom::BloomSettings}, render::color};
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
        .insert_resource(NoiseGen { gen: Source::perlin(seed), seed, noise_scale: 0.001 })
        .insert_resource(ParticleCount(NUM_PARTICLES as usize))
        .insert_resource(ColorScheme {
            background: Color::BLACK,
            particle: Color::WHITE,
            luminosity: LUMINOSITY
        })
        // .insert_resource(Luminosity(LUMINOSITY))
        .add_systems(Startup, (setup_camera, add_particles))
        .add_systems(PostStartup, draw_trails)
        .add_systems(Update, (move_particles, keyboard_input, ensure_particle_count)) // check_field, 
        .add_systems(Update, window_resize)
        .run();
}

#[derive(Resource)]
struct Luminosity(f32);

#[derive(Resource)]
struct ParticleCount(usize);

#[derive(Resource, Copy, Clone)]
struct ColorScheme {
    background: Color,
    particle: Color,
    luminosity: f32,
}

impl ColorScheme {
    fn default() -> Self {
        ColorScheme {
            background: Color::BLACK,
            particle: Color::WHITE,
            luminosity: LUMINOSITY
        }
    }

    fn custom(background: Color, particle: Color, luminosity: f32) -> Self {
        ColorScheme {
            background,
            particle,
            luminosity
        }
    }

    fn random() -> Self {
        ColorScheme {
            background: Color::rgb(rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()),
            particle: Color::rgb(rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()),
            luminosity: LUMINOSITY
        }
    }

    fn inverse(&self) -> Self {
        ColorScheme {
            background: self.particle,
            particle: self.background,
            luminosity: self.luminosity
        }
    }

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

fn add_particles(mut commands: Commands, window_size: Res<WindowSize>, particle_count: Res<ParticleCount>) {
    for _ in 0..particle_count.0 as usize {
            // add a particle
            let particle: Particle = Particle::random(window_size.width, window_size.height);

            commands.spawn(particle);
    }
}

fn ensure_particle_count(mut commands: Commands, particle_count: Res<ParticleCount>, mut particles: Query<Entity, With<Particle>>) {
    if particles.iter().count() < particle_count.0 {
        // add a particle
        let particle: Particle = Particle::random(WIDTH as f32, HEIGHT as f32);

        commands.spawn(particle);
    } else if particles.iter().count() > particle_count.0 {
        // remove a particle
        for entity in particles.iter().take(particles.iter().count() - particle_count.0) {
            commands.entity(entity).despawn_recursive();
        }
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

fn keyboard_input(mut commands: Commands, 
                  keys: Res<Input<KeyCode>>, 
                  window_size: Res<WindowSize>,
                  mut noise_gen: ResMut<NoiseGen>, 
                  mut color_scheme: ResMut<ColorScheme>,
                  mut effects: ResMut<Assets<EffectAsset>>,
                  mut particles: Query<&mut Particle>,
                  mut particle_entities: Query<Entity, With<Particle>>) {
    
    let mut should_change_particle_effects: bool = false;
    let mut new_color_scheme: ColorScheme = color_scheme.clone();
    
    if keys.just_pressed(KeyCode::Space) {
        noise_gen.seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64;
        noise_gen.gen = Source::perlin(noise_gen.seed);
    }

    if keys.just_pressed(KeyCode::Up) {
        noise_gen.noise_scale /= 2.;
    } else if keys.just_pressed(KeyCode::Down) {
        noise_gen.noise_scale *= 2.;
    }

    if keys.just_pressed(KeyCode::R) {
        // reset the particle positions and clear the effects
        should_change_particle_effects = true;
        for mut particle in particles.iter_mut() {
            particle.x = -window_size.width / 2. + rand::random::<f32>() * window_size.width;
            particle.y = -window_size.height / 2. + rand::random::<f32>() * window_size.height;
        }
    }

    if keys.just_pressed(KeyCode::I) {
        // invert the color scheme
        should_change_particle_effects = true;
        *color_scheme = color_scheme.inverse();
    }

    if keys.just_pressed(KeyCode::N) {
        // new random color scheme
        should_change_particle_effects = true;
        new_color_scheme = ColorScheme::random();
    }


    // keep these last, they modify the new color
    if keys.just_pressed(KeyCode::A) {
        // decrease the luminosity
        should_change_particle_effects = true;
        new_color_scheme.luminosity -= 1.;
        println!("Luminosity: {}", new_color_scheme.luminosity);

    } else if (keys.just_pressed(KeyCode::D)) {
        // increase the luminosity
        should_change_particle_effects = true;
        new_color_scheme.luminosity += 1.;
        println!("Luminosity: {}", new_color_scheme.luminosity);
    }

    if should_change_particle_effects {
        // commands.insert_resource(ClearColor(new_color_scheme.background.clone()));
        change_particle_effects(commands, effects, particle_entities, new_color_scheme);
        color_scheme.particle = new_color_scheme.particle;
        color_scheme.background = new_color_scheme.background;
        color_scheme.luminosity = new_color_scheme.luminosity;
    }

}

fn change_particle_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>, particles: Query<Entity, With<Particle>>, color_scheme: ColorScheme) {
    for entity in particles.iter() {
        commands.entity(entity).remove::<ParticleEffectBundle>();
    }

    add_particle_effects(commands, effects, particles, color_scheme);
}

fn add_particle_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>, particles: Query<Entity, With<Particle>>, color_scheme: ColorScheme) {
    commands.insert_resource(ClearColor(color_scheme.background));
    
    for entity in particles.iter() {

        let writer = ExprWriter::new();
        let age = writer.lit(0.0).expr();
        let init_age = SetAttributeModifier::new(Attribute::AGE, age);

        let lifetime = writer.lit(1.).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

        let particle_color: Color = color_scheme.particle;

        let luminosity: f32 = color_scheme.luminosity;
        
        let mut gradient = Gradient::new();
        gradient.add_key(0.0, Vec4::new(particle_color.r() * luminosity, particle_color.g() * luminosity, particle_color.b() * luminosity, 0.2));
        gradient.add_key(0.5, Vec4::new(particle_color.r() * luminosity, particle_color.g() * luminosity, particle_color.b() * luminosity, 0.05));
        gradient.add_key(1.0, Vec4::new(particle_color.r() * luminosity, particle_color.g() * luminosity, particle_color.b() * luminosity, 0.0));

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
            EffectAsset::new(1024, Spawner::rate(45.0.into()), writer.finish())
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

fn draw_trails(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>, particles: Query<Entity, With<Particle>>, color_scheme: Res<ColorScheme>) {
    add_particle_effects(commands, effects, particles, color_scheme.clone());
}
