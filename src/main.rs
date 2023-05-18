use macroquad::prelude::*;

use macroquad::ui::{
    hash, root_ui,
    widgets::{self, Group}
};

#[macroquad::main("UI Circuit sim")]
async fn main() {

    let texture: Texture2D = load_texture("assets/resistor.png").await.unwrap();
    let components = vec!["resistor", "voltage source", "current source", "capacitor", "inductor"];

    loop {
        clear_background(WHITE);

        widgets::Window::new(hash!(), vec2(10., 10.), vec2(320., 400.))
            .label("Components")
            .titlebar(true)
            .ui(&mut *root_ui(), |ui| {
                for component in components.iter() {
                    Group::new(hash!(component), vec2(300., 80.))
                    .ui(ui, |ui| {
                        Group::new(hash!(component, "lab"), vec2(120., 70.))
                            .ui(ui, |ui| {
                                ui.label(Vec2::new(10., 10.), component)});
                        let _drag = Group::new(hash!(component, "fig"), vec2(120., 70.))
                            .draggable(true)
                            .hoverable(true)
                            .highlight(true)
                            .ui(ui, |ui| {
                                widgets::Button::new(texture).size(vec2(100., 50.)).ui(ui);
                            });
                        
                    });

                    
                    
                }
            });


        next_frame().await;
    }
}
