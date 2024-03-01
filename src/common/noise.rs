use fastnoise_lite::*;

// https://auburn.github.io/FastNoiseLite/
pub struct FractalNoise {
    nz: FastNoiseLite,
}

impl FractalNoise {
    pub fn new(seed: i32, frequency: f32, octaves: i32) -> Self {
        let mut nz = FastNoiseLite::with_seed(seed);
        nz.set_frequency(frequency.into());
        nz.set_fractal_octaves(octaves.into());
        nz.set_noise_type(NoiseType::OpenSimplex2.into());
        nz.set_fractal_type(FractalType::FBm.into());
        Self { nz }
    }

    pub fn get_3d(&mut self, x: f32, y: f32, z: f32) -> f32 {
        (self.nz.get_noise_3d(x, y, z) + 1.) / 2.
    }

    pub fn get_2d(&mut self, x: f32, y: f32) -> f32 {
        (self.nz.get_noise_2d(x, y) + 1.) / 2.
    }
}
