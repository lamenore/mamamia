pub trait Draw {
    fn draw_to_img(&self, img: &mut image::RgbaImage);
}
