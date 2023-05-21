use macroquad::prelude::*;
use std::{collections::HashMap, vec};
use std::f32::consts;
use circuit_sim::bipoles::*;


use macroquad::ui::{
    hash, root_ui,
    widgets::{self, Group}
};


struct Node {
    position: Vec2,
    computed_id: usize
}

struct Wire {
    node1_pos: Vec2,
    node2_pos: Vec2,
    node1_id: usize,
    node2_id: usize
}

#[derive(Clone, Copy)]
enum BipoleRotation {
    AnodeUp,
    AnodeDown,
    AnodeRight,
    AnodeLeft
}


impl BipoleRotation {

    fn get_angle(&self) -> f32 {
        match self {
            Self::AnodeUp => consts::PI/2.0,
            Self::AnodeDown => 3.0/2.0 * consts::PI,
            Self::AnodeLeft => consts::PI,
            Self::AnodeRight => 0.0,
        }
    }

    fn get_rect(&self, size: Vec2, center_position: Vec2) -> Rect {
        let mut rect = match self {
            Self::AnodeUp | Self::AnodeDown => Rect { x: -size.y/2.0, y: -size.x/2.0, w: size.y, h: size.x },
            Self::AnodeLeft| Self::AnodeRight => Rect { x: -size.x/2.0, y: -size.y/2.0, w: size.x, h: size.y }
        };
        rect.x += center_position.x;
        rect.y += center_position.y;
        rect
    }

    fn get_next(&self) -> BipoleRotation {
        match self {
            Self::AnodeUp => Self::AnodeLeft,
            Self::AnodeDown => Self::AnodeRight,
            Self::AnodeLeft => Self::AnodeDown,
            Self::AnodeRight => Self::AnodeUp,
        }
    }

    fn get_matrix(angle: f32) -> Mat2 {
        mat2(vec2(angle.cos(), angle.sin()), vec2(- angle.sin(), angle.cos()))
    }
}

struct BipoleToPlace {
    size: Vec2,
    center_position: Vec2,
    rotation: BipoleRotation,
}

impl BipoleToPlace {
    fn get_anode_position(&self) -> Vec2{
        let anode_pos_rel = vec2(self.size.x/2.0, 0.0);
        let matrix = BipoleRotation::get_matrix(self.rotation.get_angle());

        matrix * anode_pos_rel + self.center_position
    }

    fn get_catode_position(&self) -> Vec2{
        let catode_pos_rel = vec2(-self.size.x/2.0, 0.0);
        let matrix = BipoleRotation::get_matrix(self.rotation.get_angle());

        matrix * catode_pos_rel + self.center_position
    }
}

struct PlacedBipole {
    name: String,
    anode_node_id: usize,
    catode_node_id: usize,
    size: Vec2,
    center_position: Vec2,
    rotation: BipoleRotation,
}

fn draw_bipole(size: Vec2, center_position: Vec2, rotation: BipoleRotation) {
    let rect = rotation.get_rect(size, center_position);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, GREEN);
    
}

trait Mode {
    fn draw(&self) {}

    fn update(&mut self, event: &ClickEvent, info: UiInfo) -> Option<Command>;
}


enum Command  {
    PlaceBipole(BipoleToPlace),
    PlaceWire{node1_id: usize, node2_id: usize, node2_pos: Vec2, is_new: bool},
    ChangeMode(Box<dyn Mode>)
}

struct ClickMode {
}

impl Mode for ClickMode {
    fn draw(&self) {
        
    }

    fn update(&mut self, event: &ClickEvent, info: UiInfo) -> Option<Command> {
        match event {
            ClickEvent::ToolbarClicked(ToolBarEvent::ArrowClicked)  => {
                None
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::PlaceClicked)  => {
                Some(Command::ChangeMode(Box::new(PlaceMode::new())))
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::WireClicked)  => {
                Some(Command::ChangeMode(Box::new(WireMode::new())))
            }
            _ => {None}



        }
    }
}

struct PlaceMode {
    bipole: BipoleToPlace,
    placed: bool,
}

impl PlaceMode {
    fn new() -> PlaceMode {
        let (x, y) = mouse_position();
        PlaceMode {
            bipole: BipoleToPlace { 
                size: vec2(50.0, 20.0), 
                rotation: BipoleRotation::AnodeUp, 
                center_position: vec2(x, y) },
            placed: false}
        }
}

impl Mode for PlaceMode {
    fn draw(&self) {

        draw_bipole(self.bipole.size, self.bipole.center_position, self.bipole.rotation);
    }

    fn update(&mut self, event: &ClickEvent, info: UiInfo) -> Option<Command>{
        let (x, y) = mouse_position();
        self.bipole.center_position = vec2(x, y);

        if let ClickEvent::ToolbarClicked(_) = event {
            let mut mode = ClickMode {};
            return mode.update(event, info);
        }

        if let ClickEvent::CanvasClicked = event {
            return Some(Command::PlaceBipole(BipoleToPlace { 
                size: self.bipole.size, 
                center_position: self.bipole.center_position, 
                rotation: self.bipole.rotation }));
        }

        if is_mouse_button_down(MouseButton::Right) {
            return Some(Command::ChangeMode(Box::new(ClickMode {})));
        }

        if let Some(KeyCode::R) = get_last_key_pressed() {
            let rotation = self.bipole.rotation;
            self.bipole.rotation = rotation.get_next();
        }

        None
        
    }
}


struct WireMode {
    drawing: bool,
    current_wire_pos1: Vec2,
    current_wire_pos2: Vec2,
    current_wire_node1_id: usize,
    current_wire_node2_id: usize
}

impl WireMode {
    fn new() -> WireMode {
        WireMode {
            drawing: false,
            current_wire_pos1: vec2(0.0, 0.0),
            current_wire_pos2: vec2(0.0, 0.0),
            current_wire_node1_id: 0,
            current_wire_node2_id: 0

        }
    }
}

impl Mode for WireMode {
    fn draw(&self) {
        if self.drawing {
            let (x, y) = mouse_position();
            let Vec2 {x: x1, y: y1} = self.current_wire_pos1;

            draw_line(x1, y1, x, y, 1.0, BLACK);
        }
    }

    fn update(&mut self, event: &ClickEvent, info: UiInfo) -> Option<Command> {

        let (x, y) = mouse_position();

        if let ClickEvent::ToolbarClicked(_) = event {
            let mut mode = ClickMode {};
            return mode.update(event, info);
        }

        if self.drawing {
            if let ClickEvent::NodeClicked { node_id } = event {
                if is_mouse_button_down(MouseButton::Left){
                    self.drawing = false;
                    self.current_wire_pos2 = vec2(x, y);
                    self.current_wire_node2_id = *node_id;
                    return Some(Command::PlaceWire { 
                        node1_id: self.current_wire_node1_id, 
                        node2_id: self.current_wire_node2_id,
                        node2_pos: self.current_wire_pos2,
                        is_new: false });
                } 
            }
            if let ClickEvent::CanvasClicked = event {
                self.current_wire_pos2 = vec2(x, y);
                let command = Some(Command::PlaceWire { 
                    node1_id: self.current_wire_node1_id, 
                    node2_id: info.current_node_id + 1,
                    node2_pos: self.current_wire_pos2,
                    is_new: true });

                self.current_wire_pos1 = vec2(x, y);
                self.current_wire_node1_id = info.current_node_id + 1;
                
                return command;
                
            }
        } else {
            if let ClickEvent::NodeClicked { node_id } = event {
                if is_mouse_button_down(MouseButton::Left) {
                    
                    self.drawing = true;
                    self.current_wire_pos1 = vec2(x, y);
                    self.current_wire_node1_id = *node_id;
                } 
            }
        }

        if is_mouse_button_down(MouseButton::Right) {
            self.drawing = false;
        }
        None
        
    }
}

struct UiData {
    nodes: HashMap<usize, Node>,
    current_node_id: usize,
    wires: HashMap<usize, Wire>,
    current_wire_id: usize,
    placed_bipoles: HashMap<String, PlacedBipole>,
    current_bipole_id: usize,
    mode: Box<dyn Mode>
}

impl UiData {

    pub fn new() -> UiData {

        let mode = ClickMode {};

        UiData { nodes: HashMap::new(), 
            current_node_id: 0, 
            wires: HashMap::new(), 
            current_wire_id: 0, 
            placed_bipoles: HashMap::new(),
            current_bipole_id: 0,
            mode: Box::new(mode)
        }
    }

    pub fn add_node(&mut self, pos: Vec2) {
        self.current_node_id += 1;
        self.nodes.insert(self.current_node_id, Node { position: pos, computed_id: 0 });
    }

    pub fn add_wire(&mut self, node1_id: usize, node2_id: usize) {
        self.current_wire_id += 1;
        self.wires.insert(self.current_wire_id, Wire { 
            node1_pos: self.nodes.get(&node1_id).unwrap().position, 
            node2_pos: self.nodes.get(&node2_id).unwrap().position, 
            node1_id, node2_id});
    }

    pub fn add_bipole(&mut self, bipole: &BipoleToPlace) {
        self.add_node(bipole.get_anode_position());
        let anode_id = self.current_node_id;

        self.add_node(bipole.get_catode_position());
        let catode_id = self.current_node_id;

        self.current_bipole_id += 1;
        let name = String::from("X") + &self.current_bipole_id.to_string();
        self.placed_bipoles.insert(name.clone(), 
            PlacedBipole { 
                name: name, 
                anode_node_id: anode_id, 
                catode_node_id: catode_id,
                size: bipole.size,
                center_position: bipole.center_position,
                rotation: bipole.rotation.clone() });
    }

    pub fn compute_bipoles(&mut self) {

    }

    pub fn is_colliding_node(&self, pos: Vec2) -> Option<usize> {
        for (id, node) in &self.nodes {
            if pos.distance(node.position) < 5.0 {
                return Some(*id)
            }
        }
        None
    }

    pub fn generate_click_event(&self, event: ToolBarEvent) -> ClickEvent{
        if !is_mouse_button_pressed(MouseButton::Left) {
            return ClickEvent::NoneClicked;
        }
        if event == ToolBarEvent::NoneClicked{
            let (x, y) = mouse_position();
            if let Some(id) = self.is_colliding_node(vec2(x, y)){
                return ClickEvent::NodeClicked { node_id: id };
            }
            return ClickEvent::CanvasClicked;
        } else {
            return ClickEvent::ToolbarClicked(event);
        }
        
    }

    pub fn update(&mut self, event: ToolBarEvent){
        
        let click_event = self.generate_click_event(event);
        let info = UiInfo {current_node_id: self.current_node_id};

        if let  Some(command) = self.mode.update(&click_event, info) {
            match command {
                Command::PlaceBipole(bipole) => {
                    self.add_bipole(&bipole);
                }
                Command::PlaceWire { node1_id, node2_id, node2_pos, is_new } => {
                    if is_new {
                        self.add_node(node2_pos);
                    }
                    self.add_wire(node1_id, node2_id);

                }
                Command::ChangeMode(mode) => {
                    self.mode = mode;
                }
            }
        }
    }

    pub fn draw(&self) {
        
        self.mode.draw();

        for (_, bipole) in &self.placed_bipoles {
            let anode = self.nodes.get(&bipole.anode_node_id).unwrap();
            let catode = self.nodes.get(&bipole.catode_node_id).unwrap();

            let (x, y) = (anode.position.x, anode.position.y);

            draw_bipole(bipole.size, bipole.center_position, bipole.rotation)
        }

        for (_, node) in &self.nodes {
            let (x, y) = (node.position.x, node.position.y);

            draw_circle(x, y, 2.0, BLACK);
        }

        for (_, wire) in &self.wires {
            let Vec2 {x: x1, y: y1} = wire.node1_pos;
            let Vec2 {x: x2, y: y2} = wire.node2_pos;

            draw_line(x1, y1, x2, y2, 1.0, BLACK);
        }
    }


}

#[derive(PartialEq, Eq)]
enum ToolBarEvent {
    WireClicked,
    ArrowClicked,
    PlaceClicked,
    NoneClicked

}

enum ClickEvent {
    ToolbarClicked(ToolBarEvent),
    NodeClicked {node_id : usize},
    BipoleClicked {name: String},
    WireClicked {node1_id: usize, node2_id: usize},
    CanvasClicked,
    NoneClicked
}

struct UiInfo {
    current_node_id: usize
}

#[macroquad::main("UI Circuit sim")]
async fn main() {

    let texture: Texture2D = load_texture("assets/resistor.png").await.unwrap();
    let components = vec!["resistor", "voltage source", "current source", "capacitor", "inductor"];
    let mut uidata = UiData::new();

    loop {
        clear_background(WHITE);

        let mut toolbar_event = ToolBarEvent::NoneClicked;

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

        widgets::Window::new(hash!(), vec2(0., 0.), vec2(320., 50.))
            .label("ToolBar")
            .titlebar(true)
            .ui(&mut *root_ui(), |ui| {
                    if ui.button(vec2(0.0, 0.0), "Click mode") {
                        toolbar_event = ToolBarEvent::ArrowClicked;
                    }
                    if ui.button(vec2(100.0, 0.0), "Place mode") {
                        toolbar_event = ToolBarEvent::PlaceClicked;
                    }
                    if ui.button(vec2(200.0, 0.0), "Wire mode") {
                        toolbar_event = ToolBarEvent::WireClicked;
                    }
                

            });

        uidata.update(toolbar_event);
        uidata.draw();
        

        next_frame().await;
    }
}
