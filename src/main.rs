use bevy::{prelude::*, window::{WindowResized, WindowResolution}, sprite::MaterialMesh2dBundle, render::view::window};
use bevy_prototype_debug_lines::{DebugLinesPlugin, DebugLines};
use na::Vector2;
use libnoise::prelude::*;

extern crate nalgebra as na;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

const VECTOR_FIELD_WIDTH: usize = 25;
const VECTOR_FIELD_HEIGHT: usize = 25;

const AMPLITUDE: f32 = 10.;
const VISUAL_AMPLITUDE: f32 = 1.;

const NUM_PARTICLES: f32 = 10000.;
const PARTICLE_RADIUS: f32 = 2.;

const SPEED_FACTOR: f32 = 1.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
               resolution: WindowResolution::new(WIDTH as f32, HEIGHT as f32),
               ..default()
            }),
            ..Default::default()
        }))
        .add_plugins(DebugLinesPlugin::default())
        .insert_resource(VectorField::new())
        .insert_resource(WindowSize {
            width: WIDTH as f32,
            height: HEIGHT as f32
        })
        .add_systems(Startup, (setup_camera, add_particles))
        .add_systems(Update, (move_particles, replace_particles)) // check_field, 
        .add_systems(Update, window_resize)
        .run();
}

#[derive(Component, Clone, Copy)]
struct Vector2D(Vector2<f32>);

impl Vector2D {
    fn new(x: f32, y: f32) -> Self {
        Vector2D(Vector2::new(x, y))
    }

    fn random() -> Self {
        Vector2D(Vector2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5))
    }

    fn get_vector(&self) -> Vector2<f32> {
        self.0
    }
}


#[derive(Component, Copy, Clone)]
struct Particle {
    x: f32,
    y: f32,
    // radius: f32,
    // color: Color,
    velocity: Vector2D,
    to_replace: bool
}

impl Particle {
    fn new(x: f32, y: f32, velocity: Vector2D) -> Self {
        Particle {
            x,
            y,
            // radius,
            // color,
            velocity: Vector2D::new(0., 0.),
            to_replace: false
        }
    }

    fn random(width: f32, height: f32) -> Self {
        let x = rand::random::<f32>() * width as f32 - (width as f32 / 2.);
        let y = rand::random::<f32>() * height as f32 - (height as f32 / 2.);

        Particle {
            x: -width as f32 / 2. + rand::random::<f32>() * 20.,
            y,
            // radius: PARTICLE_RADIUS,
            // color: Color::WHITE,
            velocity: Vector2D::random(),
            to_replace: false
        }
    }

}

#[derive(Resource)]
struct VectorField {
    field: Vec<Vector2D>
}

impl VectorField {
    fn new() -> Self {
        // (Force Vector, Position)
        let mut vec: Vec<Vector2D> = Vec::new();

        let gen: Perlin<2> = Source::perlin(41);

        for r in 0..VECTOR_FIELD_HEIGHT {
            for c in 0..VECTOR_FIELD_WIDTH {
                let noise_value = gen.sample([(10 * r) as f64 / HEIGHT as f64, (20 * c) as f64 / WIDTH as f64]) * 8. * std::f64::consts::PI;

                vec.push(
                    Vector2D::new(
                    (noise_value.cos() * AMPLITUDE as f64) as f32,
                    (noise_value.sin() * AMPLITUDE as f64) as f32));
            }
        }

        VectorField {
            field: vec
        }
    }

    fn get_vector(&self, x: usize, y: usize) -> Vector2<f32> {
        self.field[x + y * WIDTH].get_vector()
    }

    fn get_vector_from_index(&self, i: usize) -> Vector2<f32> {
        self.field[i].get_vector()
    }

    fn get_vector_world_coordinates_from_index(&self, i: usize) -> Vector2<f32> {
       let r = i / VECTOR_FIELD_WIDTH;
       let c = i % VECTOR_FIELD_WIDTH;

       self.get_vector_world_coordinates(r, c)
    }

    fn get_vector_world_coordinates(&self, r: usize, c: usize) -> Vector2<f32> {
        let x: f32 = (c as f32 / VECTOR_FIELD_WIDTH as f32) * WIDTH as f32 - (WIDTH as f32 / 2.);
        let y: f32 = (r as f32 / VECTOR_FIELD_HEIGHT as f32) * HEIGHT as f32 - (HEIGHT as f32 / 2.);

        Vector2::new(x, y)
    }

    fn get_nearest_vector(&self, x: f32, y: f32) -> Vector2<f32> {
        // find the closest vector in the field
        let mut min_dist = std::f32::MAX;
        let mut min_vec = Vector2::new(0., 0.);

        for i in 0..self.field.len() {
            let real_pos: Vector2<f32> = self.get_vector_world_coordinates_from_index(i);

            let dist = (real_pos.x - x).powi(2) + (real_pos.y - y).powi(2);

            if dist < min_dist {
                min_dist = dist;
                min_vec = self.get_vector_from_index(i);
            }
        }
        min_vec
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn check_field(field: Res<VectorField>, mut lines: ResMut<DebugLines>, window_size: Res<WindowSize>) {

    for i in 0..field.field.len() {
        let vec: Vector2<f32> = field.get_vector_from_index(i);
        let real_pos: Vector2<f32> = field.get_vector_world_coordinates_from_index(i);

        lines.line_gradient(
            Vec3::new(real_pos.x, real_pos.y, 0.),
            Vec3::new(real_pos.x + vec.x * VISUAL_AMPLITUDE, real_pos.y + vec.y * VISUAL_AMPLITUDE, 0.),
            0.,
            Color::WHITE,
            Color::RED
        );
    }
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

    // TODO: does not properly move vectors in the field
}

fn add_particles(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, window_size: Res<WindowSize>) {
    for _ in 0..NUM_PARTICLES as usize {
            // add a particle
            let particle: Particle = Particle::random(window_size.width, window_size.height);

            commands.spawn((particle, 
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(PARTICLE_RADIUS).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgba(1., 1., 1., 0.05))),
                    transform: Transform::from_translation(Vec3::new(particle.x, particle.y, 0.)),
                    ..default()
                }));
    }
}

fn replace_particles(mut commands: Commands, particles: Query<(Entity, &Particle)>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, window_size: Res<WindowSize>) {
    for (entity, particle) in particles.iter() {
        if particle.to_replace {
            commands.entity(entity).despawn();
            

            let particle: Particle = Particle::random(window_size.width, window_size.height);

            commands.spawn((particle, 
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(PARTICLE_RADIUS).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgba(1., 1., 1., 0.05))),
                    transform: Transform::from_translation(Vec3::new(particle.x, particle.y, 0.)),
                    ..default()
                }));
        }
    }
}

fn move_particles(mut particles: Query<(&mut Particle, &mut Transform)>, field: Res<VectorField>, time: Res<Time>, window_size: Res<WindowSize>) {
    for (mut particle, mut transform) in particles.iter_mut() {
        // check if the particle is out of bounds
        if (particle.x >= (window_size.width / 2f32) as f32 || particle.y >= (window_size.height / 2f32) as f32 || particle.x <= -(window_size.width / 2f32) as f32 || particle.y <= -(window_size.height / 2f32) as f32) {
            particle.to_replace = true;
            continue;
        }
        let vec = VectorField::get_nearest_vector(&field, particle.x, particle.y);
        let velocity_vec = particle.velocity.get_vector();

        particle.velocity = Vector2D::new(velocity_vec.x + (vec.x * time.delta_seconds()), velocity_vec.y + (vec.y * time.delta_seconds()));

        particle.x += particle.velocity.0.x * time.delta_seconds() * SPEED_FACTOR;
        particle.y += particle.velocity.0.y * time.delta_seconds() * SPEED_FACTOR;

        transform.translation.x = particle.x;
        transform.translation.y = particle.y;
    }
}