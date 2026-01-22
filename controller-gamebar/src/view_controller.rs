use cs2::{CS2Offset, StateCS2Handle, StateResolvedOffset};
use utils_state::{State, StateCacheType, StateRegistry};

/// View controller which transforms 3D world coordinates to 2D normalized screen coordinates (0.0-1.0)
/// This is adapted for the GameBar overlay which uses normalized coordinates
pub struct ViewController {
    view_matrix: nalgebra::Matrix4<f32>,
    /// Screen dimensions in pixels (used for aspect ratio calculation)
    pub screen_width: f32,
    pub screen_height: f32,
}

impl State for ViewController {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            view_matrix: Default::default(),
            screen_width: 1920.0,  // Default, will be updated
            screen_height: 1080.0, // Default, will be updated
        })
    }

    fn cache_type() -> StateCacheType {
        // Persistent - will be manually invalidated each frame
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let cs2 = states.resolve::<StateCS2Handle>(())?;
        let offset = states.resolve::<StateResolvedOffset>(CS2Offset::ViewMatrix)?;

        self.view_matrix = cs2.read_sized(offset.address)?;
        log::trace!("ViewController updated - view matrix first row: [{:.3}, {:.3}, {:.3}, {:.3}]", 
            self.view_matrix.m11, self.view_matrix.m12, self.view_matrix.m13, self.view_matrix.m14);
        Ok(())
    }
}

impl ViewController {
    pub fn update_screen_dimensions(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn get_camera_world_position(&self) -> Option<nalgebra::Vector3<f32>> {
        let view_matrix = self.view_matrix;
        let a = view_matrix.m22 * view_matrix.m33 - view_matrix.m32 * view_matrix.m23;
        let b = view_matrix.m32 * view_matrix.m13 - view_matrix.m12 * view_matrix.m33;
        let c = view_matrix.m12 * view_matrix.m23 - view_matrix.m22 * view_matrix.m13;
        let z = view_matrix.m11 * a + view_matrix.m21 * b + view_matrix.m31 * c;

        if z.abs() < 0.0001 {
            return None;
        }

        let d = view_matrix.m31 * view_matrix.m23 - view_matrix.m21 * view_matrix.m33;
        let e = view_matrix.m11 * view_matrix.m33 - view_matrix.m31 * view_matrix.m13;
        let f = view_matrix.m21 * view_matrix.m13 - view_matrix.m11 * view_matrix.m23;
        let g = view_matrix.m21 * view_matrix.m32 - view_matrix.m31 * view_matrix.m22;
        let h = view_matrix.m31 * view_matrix.m12 - view_matrix.m11 * view_matrix.m32;
        let k = view_matrix.m11 * view_matrix.m22 - view_matrix.m21 * view_matrix.m12;

        let x = (a * view_matrix.m41 + d * view_matrix.m42 + g * view_matrix.m43) / z;
        let y = (b * view_matrix.m41 + e * view_matrix.m42 + h * view_matrix.m43) / z;
        let z = (c * view_matrix.m41 + f * view_matrix.m42 + k * view_matrix.m43) / z;

        Some(nalgebra::Vector3::new(-x, -y, -z))
    }

    /// Transform 3D world coordinates to normalized 2D screen coordinates (0.0-1.0)
    /// Returns None if the point is behind the camera or off-screen
    pub fn world_to_screen_normalized(
        &self,
        vec: &nalgebra::Vector3<f32>,
        allow_off_screen: bool,
    ) -> Option<nalgebra::Vector2<f32>> {
        let screen_coords =
            nalgebra::Vector4::new(vec.x, vec.y, vec.z, 1.0).transpose() * self.view_matrix;

        if screen_coords.w < 0.1 {
            return None;
        }

        if !allow_off_screen
            && (screen_coords.x < -screen_coords.w
                || screen_coords.x > screen_coords.w
                || screen_coords.y < -screen_coords.w
                || screen_coords.y > screen_coords.w)
        {
            return None;
        }

        // Convert to normalized device coordinates (-1 to 1)
        let mut ndc = nalgebra::Vector2::new(
            screen_coords.x / screen_coords.w,
            screen_coords.y / screen_coords.w,
        );

        // Convert to normalized screen coordinates (0 to 1)
        ndc.x = (ndc.x + 1.0) / 2.0;
        ndc.y = (-ndc.y + 1.0) / 2.0; // Flip Y axis

        Some(ndc)
    }

    /// Calculate the 2D bounding box for a 3D object
    /// Returns (min, max) in normalized coordinates (0.0-1.0)
    pub fn calculate_box_2d(
        &self,
        vmin: &nalgebra::Vector3<f32>,
        vmax: &nalgebra::Vector3<f32>,
    ) -> Option<(nalgebra::Vector2<f32>, nalgebra::Vector2<f32>)> {
        type Vec3 = nalgebra::Vector3<f32>;
        type Vec2 = nalgebra::Vector2<f32>;

        // Get all 8 corners of the bounding box
        let points = [
            /* bottom */
            Vec3::new(vmin.x, vmin.y, vmin.z),
            Vec3::new(vmax.x, vmin.y, vmin.z),
            Vec3::new(vmin.x, vmax.y, vmin.z),
            Vec3::new(vmax.x, vmax.y, vmin.z),
            /* top */
            Vec3::new(vmin.x, vmin.y, vmax.z),
            Vec3::new(vmax.x, vmin.y, vmax.z),
            Vec3::new(vmin.x, vmax.y, vmax.z),
            Vec3::new(vmax.x, vmax.y, vmax.z),
        ];

        let mut min2d = Vec2::new(f32::MAX, f32::MAX);
        let mut max2d = Vec2::new(-f32::MAX, -f32::MAX);

        // Project all corners and find min/max
        for point in points {
            if let Some(point) = self.world_to_screen_normalized(&point, true) {
                min2d.x = min2d.x.min(point.x);
                min2d.y = min2d.y.min(point.y);

                max2d.x = max2d.x.max(point.x);
                max2d.y = max2d.y.max(point.y);
            }
        }

        if min2d.x >= max2d.x || min2d.y >= max2d.y {
            return None;
        }

        Some((min2d, max2d))
    }
}

