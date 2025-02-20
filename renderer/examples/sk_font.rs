use skia_safe::FontMgr;

fn main() {
    let font_mgr = FontMgr::new();

    let font_data = std::fs::read("/Users/marc/Library/Fonts/Lato-Regular.ttf").unwrap();
    let mut font = font_mgr.new_from_data(&font_data, None).unwrap();
    println!("{:?}", font);
}
