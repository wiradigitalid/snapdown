use snapdown_core::domain::finding::Region;

#[derive(Debug, Clone)]
pub struct MonitorGeometry {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

impl MonitorGeometry {
    pub fn new(name: String, x: i32, y: i32, width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
            scale_factor,
        }
    }

    pub fn to_physical_pixels(&self, region: &Region) -> Region {
        let px = ((region.x as f64) * self.scale_factor).round() as i32;
        let py = ((region.y as f64) * self.scale_factor).round() as i32;
        let pw = ((region.width as f64) * self.scale_factor).round() as u32;
        let ph = ((region.height as f64) * self.scale_factor).round() as u32;

        Region::new(px, py, pw, ph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_capturer_pixel_accuracy_across_mixed_dpi() {
        let monitor_standard = MonitorGeometry::new("DISPLAY1".into(), 0, 0, 1920, 1080, 1.0);
        let monitor_hidpi = MonitorGeometry::new("DISPLAY2".into(), 1920, 0, 3840, 2160, 2.0);

        let logical_region = Region::new(100, 150, 400, 300);

        let standard_pixels = monitor_standard.to_physical_pixels(&logical_region);
        assert_eq!(standard_pixels.x, 100);
        assert_eq!(standard_pixels.y, 150);
        assert_eq!(standard_pixels.width, 400);
        assert_eq!(standard_pixels.height, 300);

        let hidpi_pixels = monitor_hidpi.to_physical_pixels(&logical_region);
        assert_eq!(hidpi_pixels.x, 200);
        assert_eq!(hidpi_pixels.y, 300);
        assert_eq!(hidpi_pixels.width, 800);
        assert_eq!(hidpi_pixels.height, 600);
    }
}
