use glam::Vec3;
use hybrid_renderer::{
    assets::{
        asset_manager::AssetManager, camera::Camera, model::Model, skydome::SkydomeDescriptor,
    },
    core::{
        material::Material, mesh::Mesh, model_node::ModelNode, texture_builder::TextureDescriptor,
    },
    renderer::materials::{MaterialFactory, SpriteMaterialDescriptor},
    stage::Stage,
    util::geometry_generator::MeshUtil,
};

use hybrid_renderer::app::{AppContext, Application};
use hybrid_renderer::main_app;

use std::sync::Arc;

const CAMERA_DISTANCE: f32 = 5.0;
const ANIMATION_FPS: f32 = 30.0;
const ANIMATION_DURATION: f32 = 1.0 / ANIMATION_FPS;
const SPRITE_NODE_NAME: &str = "sprite";

struct SpriteAnimationApp {
    frame_time: f32,
    frame_count: u32,
}

impl Application for SpriteAnimationApp {
    fn init(app_context: &mut AppContext<'_>) -> (Self, Stage) {
        let size = app_context.window.inner_size();
        let camera = Camera::new(Vec3::new(0.0, 0.0, CAMERA_DISTANCE))
            .with_target(Vec3::new(0.0, 0.0, 0.0))
            .with_fov(45.0)
            .with_near(0.1)
            .with_far(1000.0)
            .with_aspect(size.width as f32 / size.height as f32);

        let stage = Stage::new(camera);

        AssetManager::new(&app_context.render_env).load_texture(
            "https://pub-227d210eb508481688aaf1983484a58a.r2.dev/flame.jpg",
            TextureDescriptor::default(),
            move |render_env, stage, texture| {
                // console::log_1(&format!("Bytes loaded: {}", bytes.len()).into());
                let sprite_material =
                    MaterialFactory::new(render_env).create_material(SpriteMaterialDescriptor {
                        texture: Arc::new(texture),
                        grid_size: (8, 8),
                    });

                let mesh_data = MeshUtil::new_procedural_quad();
                let mesh = Mesh::from_data(&render_env.render_context.device, &mesh_data);
                let mut model_node = Arc::new(ModelNode::new(
                    mesh,
                    hybrid_renderer::core::material::Material::Sprite(sprite_material),
                ));
                Arc::get_mut(&mut model_node).unwrap().name = SPRITE_NODE_NAME.to_string();
                let model = Model::new(model_node.clone(), glam::Mat4::IDENTITY, vec![]);
                stage.add_model(model);
            },
            None,
        );

        AssetManager::new(&app_context.render_env).load_skydome(
            "https://pub-227d210eb508481688aaf1983484a58a.r2.dev/modern_buildings_night_1k.exr",
            //"assets/zwartkops_curve_sunset_4k.exr",
            SkydomeDescriptor {
                radius: 150.0,
                factor: 0.95,
            },
            move |_, stage, skydome| {
                stage.set_skydome(skydome);
            },
            None,
        );

        (
            Self {
                frame_time: 0.0,
                frame_count: 0,
            },
            stage,
        )
    }

    fn update(&mut self, ctx: &mut AppContext<'_>, stage: &mut Stage, delta_time: f32) {
        self.frame_time += delta_time;

        if self.frame_time >= ANIMATION_DURATION {
            self.frame_time = 0.0;
            self.frame_count += 1;
            self.frame_count %= 8 * 8;
        }

        // TODO: models add unique_id
        for model in stage.models.iter() {
            if let Some(model_node) = model.find_node_by_name(SPRITE_NODE_NAME)
                && let Some(material) = &model_node.material
                && let Material::Sprite(sprite) = material.as_ref()
            {
                sprite.set_frame(self.frame_count, &ctx.render_env.render_context.queue);
            }
        }
    }
}

main_app!(SpriteAnimationApp, "Sprite Animation Demo");
