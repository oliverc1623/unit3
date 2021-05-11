use engine3d::{collision, events::*, geom::*, render::InstanceGroups, run, sound, Engine, DT, save_load};
use rand;
use rodio::{source::SineWave, source::Source, SpatialSink};
use std::io::BufReader;
use std::thread;
use std::time::Duration;
use winit;
use save_load::{new_save,parse_save};

const NUM_MARBLES: usize = 0;
const G: f32 = 1.0;
const SAVE_PATH: &str = "saves/save.json";


#[derive(Clone, Debug)]
pub struct Player {
    pub body: Sphere,
    pub velocity: Vec3,
    pub acc: Vec3,
    pub rot: Quat,
    pub omega: Vec3,
}

impl Player {
    const MAX_SPEED: f32 = 3.0;
    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        igs.render(
            rules.player_model,
            engine3d::render::InstanceRaw {
                model: ( Mat4::from_translation(self.body.c.to_vec()) * Mat4::from(self.rot) * Mat4::from_scale(self.body.r)).into(),
                // model: ((Mat4::from_translation(self.body.c.to_vec()) * Mat4::from_scale(self.body.r))).into(),
            },
        );
    }
    fn integrate(&mut self) {
        
        // self.velocity += ((self.rot * self.acc) + Vec3::new(0.0, -G, 0.0)) * DT;
        self.apply_impulse(Vec3::new(0.0, -G, 0.0) * DT, Vec3::zero());
        self.velocity = self.body.lin_mom;

        if self.velocity.magnitude() > Self::MAX_SPEED {
            self.velocity = self.velocity.normalize_to(Self::MAX_SPEED);
        }

        self.body.c += self.velocity * DT;

        self.omega = self.body.ang_mom; // Here we are ignoring intertia
        self.rot += 0.5 * DT * Quat::new(0.0, self.omega.x, self.omega.y, self.omega.z) * self.rot;
        
    }

    fn apply_impulse(&mut self, l: Vec3, a: Vec3) {
        self.body.lin_mom += l;
        self.body.ang_mom += a;
        // self.velocity = self.body.lin_mom;
    }

    fn new()-> Player{
        
        let mut loaded_save = save_load::parse_save(String::from(SAVE_PATH));
        match loaded_save{
        Ok(load)=>{
            let player = return Player {
                body: Sphere {
                    c: Pos3::new(load.location.x, load.location.y, load.location.z),
                    r: 0.3,
                    lin_mom: Vec3::new(0.0, 0.0, 0.0),
                    ang_mom: Vec3::new(0.0, 0.0, 0.0),
                    mass: 1.0,
                },
                velocity: Vec3::zero(),
                acc: Vec3::zero(),
                omega: Vec3::zero(),
                rot: Quat::new(1.0, 0.0, 0.0, 0.0),
            };

        }
        Err(_)=>{ println!("No save starting new game");
        }
    }
    return Player {
        body: Sphere {
            c: Pos3::new(0.0, 3.0, 0.0),
            r: 0.3,
            lin_mom: Vec3::new(0.0, 0.0, 0.0),
            ang_mom: Vec3::new(0.0, 0.0, 0.0),
            mass: 1.0,
        },
        velocity: Vec3::zero(),
        acc: Vec3::zero(),
        omega: Vec3::zero(),
        rot: Quat::new(1.0, 0.0, 0.0, 0.0),
    };
}
}

trait Camera {
    fn new() -> Self;
    fn update(&mut self, _events: &engine3d::events::Events, _player: &Player) {}
    fn render(&self, _rules: &GameData, _igs: &mut InstanceGroups) {}
    fn update_camera(&self, _cam: &mut engine3d::camera::Camera) {}
    fn integrate(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct FPCamera {
    pub pitch: f32,
    player_pos: Pos3,
    player_rot: Quat,
}

impl Camera for FPCamera {
    fn new() -> Self {
        Self {
            pitch: 0.0,
            player_pos: Pos3::new(0.0, 0.0, 0.0),
            player_rot: Quat::new(1.0, 0.0, 0.0, 0.0),
        }
    }
    fn update(&mut self, events: &engine3d::events::Events, player: &Player) {
        let (_dx, dy) = events.mouse_delta();
        self.pitch += dy / 100.0;
        self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);
        self.player_pos = player.body.c;
        self.player_rot = player.rot;
    }
    fn update_camera(&self, c: &mut engine3d::camera::Camera) {
        c.eye = self.player_pos + Vec3::new(0.0, 0.5, 0.0);
        // The camera is pointing at a point just in front of the composition of the player's rot and the camera's rot (player * cam * forward-offset)
        let rotation = self.player_rot
            * (Quat::from(cgmath::Euler::new(
                cgmath::Rad(self.pitch),
                cgmath::Rad(0.0),
                cgmath::Rad(0.0),
            )));
        let offset = rotation * Vec3::unit_z();
        c.target = c.eye + offset;
    }
}

#[derive(Clone, Debug)]
pub struct OrbitCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    player_pos: Pos3,
    player_rot: Quat,
}

impl Camera for OrbitCamera {
    fn new() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            distance: 5.0,
            player_pos: Pos3::new(0.0, 0.0, 0.0),
            player_rot: Quat::new(1.0, 0.0, 0.0, 0.0),
        }
    }
    fn update(&mut self, events: &engine3d::events::Events, player: &Player) {
        let (dx, dy) = events.mouse_delta();
        self.pitch += dy / 100.0;
        self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);

        self.yaw += dx / 100.0;
        self.yaw = self.yaw.clamp(-PI / 4.0, PI / 4.0);
        if events.key_pressed(KeyCode::Up) {
            self.distance -= 0.5;
        }
        if events.key_pressed(KeyCode::Down) {
            self.distance += 0.5;
        }
        self.player_pos = player.body.c;
        self.player_rot = player.rot;
        // TODO: when player moves, slightly move yaw towards zero
    }
    fn update_camera(&self, c: &mut engine3d::camera::Camera) {
        // The camera should point at the player
        c.target = self.player_pos;
        // And rotated around the player's position and offset backwards

        // let camera_rot = self.player_rot
        //     * Quat::from(cgmath::Euler::new(
        //         cgmath::Rad(self.pitch),
        //         cgmath::Rad(self.yaw),
        //         cgmath::Rad(0.0),
        //     ));
        // let offset = camera_rot * Vec3::new(0.0, 0.0, -self.distance);
        // c.eye = self.player_pos + offset;

        // To be fancy, we'd want to make the camera's eye to be an object in the world and whose rotation is locked to point towards the player, and whose distance from the player is locked, and so on---so we'd have player OR camera movements apply accelerations to the camera which could be "beaten" by collision.
    }
}

#[derive(Clone, Debug)]
pub struct TopDownCamera {
    pub distance: f32,
    player_pos: Pos3,
}

impl Camera for TopDownCamera {
    fn new() -> Self {
        Self {
            distance: 15.0,
            player_pos: Pos3::new(0.0, 0.0, 0.0),
        }
    }
    fn update(&mut self, events: &engine3d::events::Events, player: &Player) {
        if events.key_pressed(KeyCode::Up) {
            self.distance -= 0.5;
        }
        if events.key_pressed(KeyCode::Down) {
            self.distance += 0.5;
        }
        self.player_pos = player.body.c;
    }
    fn update_camera(&self, c: &mut engine3d::camera::Camera) {
        // The camera should point at the player
        c.target = self.player_pos;
        let offset = Vec3::new(0.0, self.distance, 0.001);
        c.eye = self.player_pos + offset;
        // To be fancy, we'd want to make the camera's eye to be an object in the world and whose rotation is locked to point towards the player, and whose distance from the player is locked, and so on---so we'd have player OR camera movements apply accelerations to the camera which could be "beaten" by collision.
    }
}

#[derive(Clone, Debug)]
pub struct Marbles {
    pub body: Vec<Sphere>,
    pub velocity: Vec<Vec3>,
}

impl Marbles {
    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        igs.render_batch(
            rules.marble_model,
            self.body.iter().map(|body| engine3d::render::InstanceRaw {
                model: (Mat4::from_translation(body.c.to_vec()) * Mat4::from_scale(body.r)).into(),
            }),
        );
    }
    fn integrate(&mut self) {
        for vel in self.velocity.iter_mut() {
            *vel += Vec3::new(0.0, -G, 0.0) * DT;
        }
        for (body, vel) in self.body.iter_mut().zip(self.velocity.iter()) {
            body.c += vel * DT;
        }
    }
    fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Sphere, &mut Vec3)> {
        self.body.iter_mut().zip(self.velocity.iter_mut())
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wall {
    pub body: Plane,
    control: (i8, i8),
}

impl Wall {
    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        igs.render(
            rules.wall_model,
            engine3d::render::InstanceRaw {
                model: (Mat4::from(cgmath::Quaternion::between_vectors(
                    Vec3::new(0.0, 1.0, 0.0),
                    self.body.n,
                )) * Mat4::from_translation(Vec3::new(0.0, -0.025, 0.0))
                    * Mat4::from_nonuniform_scale(0.5, 0.05, 0.5))
                .into(),
            },
        );
    }

    fn input(&mut self, events: &engine3d::events::Events) {
        self.control.0 = if events.key_held(KeyCode::A) {
            -1
        } else if events.key_held(KeyCode::D) {
            1
        } else {
            0
        };
        self.control.1 = if events.key_held(KeyCode::W) {
            -1
        } else if events.key_held(KeyCode::S) {
            1
        } else {
            0
        };
    }
    fn integrate(&mut self) {
        self.body.n += Vec3::new(
            self.control.0 as f32 * 0.4 * DT,
            0.0,
            self.control.1 as f32 * 0.4 * DT,
        );
        self.body.n = self.body.n.normalize();
    }
}

struct Cube {
    pub body: AABB,
    pub velocity: Vec3,
}

impl Cube {
    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        let scale = self.body.half_sizes * 2.0;
        igs.render(
            rules.box_model,
            engine3d::render::InstanceRaw {
                model: (Mat4::from_translation(self.body.c.to_vec())
                    * Mat4::from_nonuniform_scale(scale.x, scale.y, scale.y))
                .into(),
            },
        );
    }

    fn integrate(&mut self) {
        self.body.c += self.velocity * DT;
    }
}

struct Game {
    marbles: Marbles,
    cubes: Vec<Cube>,
    wall: Wall,
    player: Player,
    td_player: Player,
    camera: OrbitCamera,
    alt_camera: TopDownCamera,
    pm: Vec<collision::Contact<usize>>,
    pw: Vec<collision::Contact<usize>>,
    mm: Vec<collision::Contact<usize>>,
    mw: Vec<collision::Contact<usize>>,
    pb: Vec<collision::Contact<usize>>,
    use_alt_cam: bool,
    // sound: sound::Sound,
}
struct GameData {
    marble_model: engine3d::assets::ModelRef,
    box_model: engine3d::assets::ModelRef,
    wall_model: engine3d::assets::ModelRef,
    player_model: engine3d::assets::ModelRef,
    // sound: sound::Sound,
}

impl engine3d::Game for Game {
    type StaticData = GameData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        use rand::Rng;

        
        let player = Player::new();

        let wall = Wall {
            body: Plane {
                n: Vec3::new(0.0, 1.0, 0.0),
                d: 0.0,
            },
            control: (0, 0),
        };
        let player = Player {
            body: Sphere {
                c: Pos3::new(10.0, 0.3, 10.0),
                r: 0.3,
                lin_mom: Vec3::new(0.0, 0.0, 0.0),
                ang_mom: Vec3::new(0.0, 0.0, 0.0),
                mass: 1.0,
            },
            velocity: Vec3::zero(),
            acc: Vec3::zero(),
            omega: Vec3::zero(),
            rot: Quat::new(1.0, 0.0, 0.0, 0.0),
        };
        let td_player = player.clone();
        let camera = OrbitCamera::new();
        let alt_camera = TopDownCamera::new();
        let mut rng = rand::thread_rng();
        let marbles = Marbles {
            body: (0..NUM_MARBLES)
                .map(move |_x| {
                    let x = rng.gen_range(-5.0..5.0);
                    let y = rng.gen_range(1.0..5.0);
                    let z = rng.gen_range(-5.0..5.0);
                    let r = rng.gen_range(0.1..1.0);
                    Sphere {
                        c: Pos3::new(x, y, z),
                        r,
                        lin_mom: Vec3::new(0.0, 0.0, 0.0),
                        ang_mom: Vec3::new(0.0, 0.0, 0.0),
                        mass: 1.0,
                    }
                })
                .collect::<Vec<_>>(),
            velocity: vec![Vec3::zero(); NUM_MARBLES],
        };

        let b = AABB {
            c: Pos3::new(22.0, 1.0, 22.0),
            // axes: Mat3::new(200.0, 200.0, 0.0, 0.0, 200.0, 0.0, 0.0, 0.0, 200.0),
            half_sizes: Vec3::new(0.75, 0.75, 0.75),
        };
        let b2 = AABB {
            c: Pos3::new(1.0, 1.0, 35.0),
            // axes: Mat3::new(200.0, 200.0, 0.0, 0.0, 200.0, 0.0, 0.0, 0.0, 200.0),
            half_sizes: Vec3::new(15.0, 5.0, 1.0),
        };
        let b3 = AABB {
            c: Pos3::new(35.0, 1.0, 1.0),
            // axes: Mat3::new(200.0, 200.0, 0.0, 0.0, 200.0, 0.0, 0.0, 0.0, 200.0),
            half_sizes: Vec3::new(1.0, 15.0, 15.0),
        };
        let b4 = AABB {
            c: Pos3::new(-35.0, 1.0, 1.0),
            // axes: Mat3::new(200.0, 200.0, 0.0, 0.0, 200.0, 0.0, 0.0, 0.0, 200.0),
            half_sizes: Vec3::new(1.0, 15.0, 15.0),
        };
        let b5 = AABB {
            c: Pos3::new(1.0, 1.0, -35.0),
            // axes: Mat3::new(200.0, 200.0, 0.0, 0.0, 200.0, 0.0, 0.0, 0.0, 200.0),
            half_sizes: Vec3::new(15.0, 5.0, 1.0),
        };

        let cubes = vec![Cube {body: b, velocity:Vec3::zero()}, 
                        Cube {body: b2, velocity: Vec3::zero()}, 
                        Cube{body: b3, velocity: Vec3::zero()},
                        Cube{body: b4, velocity: Vec3::zero()},
                        Cube{body: b5, velocity: Vec3::zero()}];
        // let cubes = vec![];
        let wall_model = engine.load_model("floor.obj");
        let marble_model = engine.load_model("sphere.obj");
        let player_model = engine.load_model("sphere.obj");
        let box_model = engine.load_model("cube.obj");
        (
            Self {
                // camera_controller,
                marbles,
                wall,
                cubes,
                player,
                td_player,
                camera,
                alt_camera,
                // TODO nice this up somehow
                mm: vec![],
                mw: vec![],
                pm: vec![],
                pw: vec![],
                pb: vec![],
                use_alt_cam: false,
            },
            GameData {
                wall_model,
                marble_model,
                box_model,
                player_model,
            },
        )
    }
    fn render(
        &mut self,
        rules: &Self::StaticData,
        assets: &engine3d::assets::Assets,
        igs: &mut InstanceGroups,
    ) {
        self.wall.render(rules, igs);
        self.marbles.render(rules, igs);
        self.player.render(rules, igs);
        self.cubes.iter().for_each(|c| c.render(rules, igs));
        // self.camera.render(rules, igs);
    }
    fn update(&mut self, _rules: &Self::StaticData, engine: &mut Engine, sound: &sound::Sound) {
        // dbg!(self.player.body);
        // TODO update player acc with controls
        // TODO update camera with controls/player movement
        // TODO TODO show how spherecasting could work?  camera pseudo-entity collision check?  camera entity for real?
        // self.camera_controller.update(engine);
        if self.player.body.touching(&self.cubes[0].body) {
            println!("touching emitting box");
            self.cubes[0].velocity = Vec3::new(0.0, -1.0, 0.0);
            self.cubes[1].velocity = Vec3::new(0.0, -1.0, 0.0);
        }

        self.player.acc = Vec3::zero();
        if self.use_alt_cam {
            if engine.events.key_held(KeyCode::W) {
                self.td_player.body.c.z += -0.1;
            } else if engine.events.key_held(KeyCode::S) {
                self.td_player.body.c.z += 0.1;
            }

            if engine.events.key_held(KeyCode::A) {
                self.td_player.body.c.x += -0.1;
            } else if engine.events.key_held(KeyCode::D) {
                self.td_player.body.c.x += 0.1;
            }
        } else {
            if engine.events.key_held(KeyCode::W) {
                self.player
                    .apply_impulse(Vec3::new(0.0, 0.0, 0.1), Vec3::new(0.1, 0.0, 0.0));
            } else if engine.events.key_held(KeyCode::S) {
                self.player
                    .apply_impulse(Vec3::new(0.0, 0.0, -0.1), Vec3::new(-0.1, 0.0, 0.0));
            }

            if engine.events.key_held(KeyCode::A) {
                self.player
                    .apply_impulse(Vec3::new(0.1, 0.0, 0.0), Vec3::new(0.0, 0.0, -0.1));
            } else if engine.events.key_held(KeyCode::D) {
                self.player
                    .apply_impulse(Vec3::new(-0.1, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.1));
            }
            if self.player.body.lin_mom.magnitude2() > 1.3 {
                self.player.body.lin_mom = self.player.body.lin_mom.normalize_to(1.3);
            }
            if self.player.body.ang_mom.magnitude2() > 1.3 {
                self.player.body.ang_mom = self.player.body.ang_mom.normalize_to(1.3);
            }

            if engine.events.key_held(KeyCode::Q) {
                self.player.omega = Vec3::unit_y();
            } else if engine.events.key_held(KeyCode::E) {
                self.player.omega = -Vec3::unit_y();
            } else {
                self.player.omega = Vec3::zero();
            }
        }
        if engine.events.key_pressed(KeyCode::C) {
            self.use_alt_cam = !self.use_alt_cam;
            self.td_player = self.player.clone();
        }
        if engine.events.key_pressed(KeyCode::X) {
           save_load::new_save(self.player.body.c.clone(),String::from(SAVE_PATH));
        }

        if self.use_alt_cam {
            self.alt_camera.update(&engine.events, &self.td_player);
            //self.td_player.integrate();
        } else {
            self.camera.update(&engine.events, &self.player);
            self.player.integrate();
        }

        // self.walls.integrate();
        self.player.integrate();
        self.marbles.integrate();
        self.cubes.iter_mut().for_each(|b| b.integrate());
        if self.use_alt_cam {
            self.alt_camera.integrate();
        } else {
            self.camera.integrate();
        }

        {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            for (body, vel) in self.marbles.iter_mut() {
                if (body.c.distance(Pos3::new(0.0, 0.0, 0.0))) >= 40.0 {
                    body.c = Pos3::new(
                        rng.gen_range(-5.0..5.0),
                        rng.gen_range(1.0..5.0),
                        rng.gen_range(-5.0..5.0),
                    );
                    *vel = Vec3::zero();
                }
            }
        }
        self.mm.clear();
        self.mw.clear();
        self.pm.clear();
        self.pw.clear();
        self.pb.clear();
        let mut pb = [self.player.body];
        let mut pv = [self.player.velocity];
        collision::gather_contacts_ab(&pb, &self.marbles.body, &mut self.pm);
        collision::gather_contacts_ab(&pb, &[self.wall.body], &mut self.pw);
        collision::gather_contacts_ab(&pb, &[self.cubes[1].body], &mut self.pb);
        collision::gather_contacts_ab(&self.marbles.body, &[self.wall.body], &mut self.mw);
        collision::gather_contacts_aa(&self.marbles.body, &mut self.mm);
        collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.wall.body], &mut self.pw);
        collision::restitute_dyn_stat(
            &mut self.marbles.body,
            &mut self.marbles.velocity,
            &[self.wall.body],
            &mut self.mw,
        );
        collision::restitute_dyn_stat(
            &mut pb,
            &mut pv,
            &[self.cubes[1].body],
            &mut self.pb,
        );
        collision::restitute_dyns(
            &mut self.marbles.body,
            &mut self.marbles.velocity,
            &mut self.mm,
        );
        collision::restitute_dyn_dyn(
            &mut pb,
            &mut pv,
            &mut self.marbles.body,
            &mut self.marbles.velocity,
            &mut self.pm,
        );
        self.player.body = pb[0];
        self.player.velocity = pv[0];

        for collision::Contact { a: ma, .. } in self.mw.iter() {
            // apply "friction" to marbles on the ground
            self.marbles.velocity[*ma] *= 0.995;
        }
        for collision::Contact { a: pa, .. } in self.pw.iter() {
            // apply "friction" to players on the ground
            assert_eq!(*pa, 0);
            self.player.body.lin_mom *= 0.98;
            self.player.body.ang_mom *= 0.98;
        }

        if self.use_alt_cam {
            self.alt_camera.update_camera(engine.camera_mut());
        } else {
            self.camera.update_camera(engine.camera_mut());
        }
        // play sound
        if engine.events.key_pressed(KeyCode::H) {
            sound.sink.play();
            let cube_pos = self.cubes[0].body.c;
            println!("cubex pos: {}", cube_pos[0]);
            println!("cube z pos: {}", cube_pos[2]);
            println!("my x pos: {}", self.player.body.c[0]);
            println!("my z pos: {}", self.player.body.c[2]);

            let x_diff = cube_pos[0] - self.player.body.c[0];
            let z_diff = cube_pos[2] - self.player.body.c[2];
            // top right
            if x_diff > 0.0 && z_diff < 0.0 {
                // we are to right of cube
                println!("xdip: {}", x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_left_to_right(x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_bottom_to_top(z_diff);
            }
            // bottom left
            if x_diff > 0.0 && z_diff > 0.0 {
                // we are to right of cube
                println!("xdip: {}", x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_left_to_right(x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_top_to_bottom(z_diff);
            }
            // top right
            if x_diff < 0.0 && z_diff > 0.0 {
                // we are to right of cube
                println!("xdip: {}", x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_right_to_left(x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_top_to_bottom(z_diff);
            }
            // bottom right
            if x_diff < 0.0 && z_diff < 0.0 {
                // we are to right of cube
                println!("xdip: {}", x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_right_to_left(x_diff);
                sound.add_sound("content/beep3.ogg");
                sound.play_top_to_bottom(z_diff);
            }
        }
        sound.sink.pause();
    }
}

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    run::<GameData, Game>(window, std::path::Path::new("content"));
}
