use engine3d::{events::*, geom::*, render::InstanceGroups, run, Engine, DT, anim::Bone};
use winit;

#[derive(Clone, Debug)]
pub struct Player {
    pos: Pos3,
    bones: Vec<Bone>,
    t:f32,
    anim:usize
}

impl Player {
    fn render(&mut self, rules: &GameData, assets:&engine3d::assets::Assets, igs: &mut InstanceGroups) {
        let anim = assets.get_anim(rules.player_anims[self.anim]).unwrap();
        let rig = assets.get_rig(rules.player_rig).unwrap();
        rig.reset(&mut self.bones);
        anim.sample(self.t, rig, &mut self.bones);
        igs.render_anim(
            rules.player_model,
            engine3d::render::InstanceRaw {
                model: (Mat4::from_translation(self.pos.to_vec()) *
                        Mat4::from_scale(0.01)).into(),
            },
            self.bones.clone(),
        );
    }
    fn integrate(&mut self, rules:&GameData) {
        self.t += DT;
        if self.t > 4.0 {
            self.t = 0.0;
            self.anim += 1;
            self.anim = self.anim % rules.player_anims.len();
            println!("Switch to anim {}",self.anim);
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrbitCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    player_pos: Pos3,
}

impl OrbitCamera {
    fn new() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            distance: 5.0,
            player_pos: Pos3::new(0.0, 0.0, 0.0),
        }
    }
    fn update(&mut self, events: &engine3d::events::Events, player: &Player) {
        let (dx,dy) = events.mouse_delta();
        self.pitch += dy / 100.0;
        self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);

        self.yaw += dx / 100.0;
        self.yaw = self.yaw.clamp(-PI, PI);
        if events.key_held(KeyCode::Up) {
            self.distance -= 0.5;
        }
        if events.key_held(KeyCode::Down) {
            self.distance += 0.5;
        }
        self.player_pos = player.pos;
        // TODO: when player moves, slightly move yaw towards zero
    }
    fn update_camera(&self, c: &mut engine3d::camera::Camera) {
        // The camera should point at the player
        c.target = self.player_pos;
        // And rotated around the player's position and offset backwards
        let camera_rot = Quat::from(cgmath::Euler::new(
            cgmath::Rad(0.0),
            cgmath::Rad(self.yaw),
            cgmath::Rad(0.0),
        ));
        let camera_rot = camera_rot*Quat::from(cgmath::Euler::new(
            cgmath::Rad(self.pitch),
            cgmath::Rad(0.0),
            cgmath::Rad(0.0),
        ));
        let offset = camera_rot * Vec3::new(0.0, 0.0, -self.distance);
        c.eye = self.player_pos + offset;
        // To be fancy, we'd want to make the camera's eye to be an object in the world and whose rotation is locked to point towards the player, and whose distance from the player is locked, and so on---so we'd have player OR camera movements apply accelerations to the camera which could be "beaten" by collision.
    }
}

struct Game {
    player: Player,
    camera: OrbitCamera,
}
struct GameData {
    player_model: engine3d::assets::ModelRef,
    player_rig: engine3d::assets::RigRef,
    player_anims: Vec<engine3d::assets::AnimRef>,
}

impl engine3d::Game for Game {
    type StaticData = GameData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        let player = Player {
            pos: Pos3::new(0.0, 5.0, 0.0),
            bones: vec![engine3d::anim::Bone::default(); engine3d::render::BONE_MAX],
            t:0.0,
            anim:0
        };
        let (player_models, player_rigs, player_anims) =
            engine.load_gltf("khronos/Fox/glTF/Fox.gltf");
        (
            Self {
                player,
                camera: OrbitCamera::new(),
            },
            GameData {
                player_model: player_models[0],
                player_rig: player_rigs[0],
                player_anims,
            },
        )
    }
    fn render(&mut self, rules: &Self::StaticData, assets:&engine3d::assets::Assets, igs: &mut InstanceGroups) {
        self.player.render(rules, assets, igs);
    }
    fn update(&mut self, rules: &Self::StaticData, engine: &mut Engine) {
        self.player.integrate(rules);
        self.camera.update(&engine.events, &self.player);
        self.camera.update_camera(engine.camera_mut());
    }
}
fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    run::<GameData, Game>(window, std::path::Path::new("content"));
}
