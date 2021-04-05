use rltk::{Rltk, RGB};
use specs::prelude::*;

use crate::components::{ParticleLifetime, Position, Renderable};

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime_ms: f32,
}

pub struct ParticlesBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticlesBuilder {
    pub fn new() -> ParticlesBuilder {
        ParticlesBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        lifetime_ms: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime_ms,
        })
    }
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticlesBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut particles, mut particles_builder) = data;
        for particle_req in particles_builder.requests.iter() {
            let particle = entities.create();
            positions
                .insert(
                    particle,
                    Position {
                        x: particle_req.x,
                        y: particle_req.y,
                    },
                )
                .expect("Unable to insert position");
            renderables
                .insert(
                    particle,
                    Renderable {
                        glyph: particle_req.glyph,
                        fg: particle_req.fg,
                        bg: particle_req.bg,
                        render_order: 0,
                    },
                )
                .expect("Unable to insert renderable");
            particles
                .insert(
                    particle,
                    ParticleLifetime {
                        lifetime_ms: particle_req.lifetime_ms,
                    },
                )
                .expect("Unable to inser lifetime");
        }

        particles_builder.requests.clear();
    }
}

pub fn cull_dead_particles(world: &mut World, ctx: &Rltk) {
    let mut dead_particles: Vec<Entity> = Vec::new();

    {
        let mut particles = world.write_storage::<ParticleLifetime>();
        let entities = world.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(entity);
            }
        }
    }

    for particle in dead_particles.iter() {
        world
            .delete_entity(*particle)
            .expect("Particle will not die");
    }
}
