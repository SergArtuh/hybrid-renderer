use hybrid_renderer::assets::model::Model;
use hybrid_renderer::core::model_node::ModelNode;
use hybrid_renderer::main_app;
use hybrid_renderer::renderer::materials::MaterialFactory;
use std::sync::Arc;

use hybrid_renderer::app::{AppContext, Application};
use hybrid_renderer::assets::camera::Camera;
use hybrid_renderer::core::math::Vec3;
use hybrid_renderer::core::mesh::Mesh;
use hybrid_renderer::stage::Stage;
use hybrid_renderer::util::geometry_generator::MeshUtil;
const CAMERA_DISTANCE: f32 = 5.0;
struct SimpleSphereApp {}

impl Application for SimpleSphereApp {
    fn init(app_context: &mut AppContext<'_>) -> (Self, Stage) {
        let size = app_context.window.inner_size();
        let camera = Camera::new(Vec3::new(0.0, 0.0, CAMERA_DISTANCE))
            .with_target(Vec3::new(0.0, 0.0, 0.0))
            .with_fov(45.0)
            .with_aspect(size.width as f32 / size.height as f32);

        let mesh_data = MeshUtil::new_sphere(0.8, 64, 64);
        let mesh = Mesh::from_data(&app_context.render_env.render_context.device, &mesh_data);

        let material = MaterialFactory::new(&app_context.render_env).create_default_material();
        let model_node = ModelNode::new(mesh, material);
        let model = Model::new(Arc::new(model_node), glam::Mat4::IDENTITY);

        let mut stage = Stage::new(camera);
        stage.add_model(model);

        (Self {}, stage)
    }
}

main_app!(SimpleSphereApp, "Simple Sphere Demo");
