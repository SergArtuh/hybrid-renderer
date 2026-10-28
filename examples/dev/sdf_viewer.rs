use hybrid_renderer::assets::asset_manager::{AssetLoadingGroup, AssetManager};
use hybrid_renderer::assets::model::Model;
use hybrid_renderer::core::sdf_volume::SdfVolume;
use hybrid_renderer::core::texture_builder::{
    ComponentPrecision, TextureChannels, TextureDescriptor, TextureFormatDescriptor,
};
use hybrid_renderer::core::uniforms::SDFUniform;
use hybrid_renderer::input::camera_controller::{
    CameraController, OrbitCameraController, OrbitCameraControllerDescriptor,
};

use std::sync::Arc;

use hybrid_renderer::assets::camera::Camera;
use hybrid_renderer::core::math::Vec3;
use hybrid_renderer::stage::Stage;

use hybrid_renderer::assets::skydome::SkydomeDescriptor;

use hybrid_renderer::app::{AppContext, Application};
use hybrid_renderer::main_app;

const CAMERA_DISTANCE: f32 = 5.0;
//const STAGE_MODEL_NAME: &str = "porsche";
const STAGE_MODEL_NAME: &str = "truck";

struct SdfViewerApp {}

impl Application for SdfViewerApp {
    fn init(app_context: &mut AppContext<'_>) -> (Self, Stage) {
        let size = app_context.window.inner_size();
        let camera = Camera::new(Vec3::new(0.0, 1., CAMERA_DISTANCE))
            .with_target(Vec3::ZERO)
            .with_fov(60.0)
            .with_near(0.1)
            .with_far(1000.0)
            .with_aspect(size.width as f32 / size.height as f32);

        let camera_controller = OrbitCameraController::new(OrbitCameraControllerDescriptor {
            target: Vec3::ZERO,
            distance: CAMERA_DISTANCE,
            start_pitch: 0.15,
            sensitivity: 0.2,
            pinch_zoom_speed: 0.1,
            start_yaw: 2.0 * std::f32::consts::PI / 3.0,
            ..Default::default()
        });

        let mut stage = Stage::new(camera);
        stage.set_camera_controller(CameraController::Orbit(camera_controller));

        let loading_group = AssetLoadingGroup::create_parallel(&mut app_context.render_env);

        AssetManager::new(&app_context.render_env).load_skydome(
            "assets/zwartkops_curve_sunset_4k.exr",
            //"https://pub-227d210eb508481688aaf1983484a58a.r2.dev/zwartkops_curve_sunset_4k.exr",
            SkydomeDescriptor {
                radius: 150.0,
                factor: 0.95,
            },
            move |_, stage, skydome| {
                println!("First skydome loaded");
                stage.set_skydome(skydome);
            },
            Some(Arc::clone(&loading_group)),
        );

        AssetManager::new(&app_context.render_env).load_model(
            //"assets/models/porsche.glb",
            // "assets/models/box.glb",
            format!("assets/models/{}.glb", STAGE_MODEL_NAME),
            //"https://pub-227d210eb508481688aaf1983484a58a.r2.dev/porsche.glb",
            move |_, stage, asset_models| {
                for model_node in &asset_models.scene_roots {
                    let mut model =
                        Model::new(Arc::clone(model_node), glam::Mat4::IDENTITY, vec![]);
                    //model.transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.6, 0.0));
                    model.name = STAGE_MODEL_NAME.to_string();

                    println!("Second model loaded");

                    // Remove the fake shadows in the floor
                    if let Some(node) = model.find_node_by_name("Plane") {
                        node.is_visible.set(false);
                    }

                    stage.add_model(model);
                }
            },
            Some(Arc::clone(&loading_group)),
        );

        AssetManager::new(&app_context.render_env).load_sdf_from_texture(
            // "assets/models/box_sdf_64_64_64.exr",
            // "assets/models/box_sdf_64_64_64.json",
            format!("assets/models/{}.exr", STAGE_MODEL_NAME),
            format!("assets/models/{}.json", STAGE_MODEL_NAME),
            //"assets/models/car_sdf_64x64x64.exr",
            //"https://pub-227d210eb508481688aaf1983484a58a.r2.dev/zwartkops_curve_sunset_4k.exr",
            move |_, stage, sdf_volume| {
                let model = stage
                    .models
                    .iter_mut()
                    .find(|model| model.name == STAGE_MODEL_NAME)
                    .unwrap();
                model.sdf_volumes.push(sdf_volume);
            },
            Some(Arc::clone(&loading_group)),
        );

        (Self {}, stage)
    }
}

main_app!(SdfViewerApp, "Sdf Viewer");
