use macroquad::prelude::*;
use mathru::algebra::linear::Vector;
use mathru::elementary::Power;
use std::{collections::HashMap, vec};
use std::f32::consts;
use circuit_sim::bipoles;
use circuit_sim::plotter::PlotIterator;


use macroquad::ui::{
    hash, root_ui,
    widgets::{self, Group}
};

trait BipoleFactory {
    fn set_parameter(&mut self, name: &str, value: f64);

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour>;

    fn get_parameters(&self) -> HashMap<String, f64>;

}

struct VoltageSourceFactory {
    value: f64
}

impl BipoleFactory for VoltageSourceFactory {
    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "value" {
            self.value = value;
        }
    }

    fn get_parameters(&self) -> HashMap<String, f64> {
        HashMap::from([(String::from("value"), self.value)])
    }

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour> {
        Box::new(bipoles::VoltageSource::new(self.value))
    }
}

struct ResistorFactory {
    resistance: f64
}

impl BipoleFactory for ResistorFactory {
    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "resistance" {
            self.resistance = value;
        }
    }

    fn get_parameters(&self) -> HashMap<String, f64> {
        HashMap::from([(String::from("resistance"), self.resistance)])
    }

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour> {
        Box::new(bipoles::Resistor::new(self.resistance))
    }
}

struct CapacitorFactory {
    capacitance: f64
}

impl BipoleFactory for CapacitorFactory {
    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "capacitance" {
            self.capacitance = value;
        }
    }

    fn get_parameters(&self) -> HashMap<String, f64> {
        HashMap::from([(String::from("capcitance"), self.capacitance)])
    }

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour> {
        Box::new(bipoles::Capacitor::new(self.capacitance, 0.0))
    }
}


struct InductorFactory {
    induttance: f64
}

impl BipoleFactory for InductorFactory {
    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "induttance" {
            self.induttance = value;
        }
    }

    fn get_parameters(&self) -> HashMap<String, f64> {
        HashMap::from([(String::from("induttance"), self.induttance)])
    }

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour> {
        Box::new(bipoles::Inductor::new(self.induttance, 0.0))
    }
}

struct DiodeFactory {
    current_s: f64,
    voltage_vt: f64,
}

impl BipoleFactory for DiodeFactory {
    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "is" {
            self.current_s = value;
        } else if name == "vt" {
            self.voltage_vt = value;
        }
    }

    fn get_parameters(&self) -> HashMap<String, f64> {
        HashMap::from([(String::from("is"), self.current_s), (String::from("vt"), self.voltage_vt)])
    }

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour> {
        Box::new(bipoles::Diode::new(self.current_s, self.voltage_vt, 1.08, 0.9))
    }
}

struct SinusoidalVoltageSourceFactory {
    value: f64,
    frequency_hz: f64,
}

impl BipoleFactory for SinusoidalVoltageSourceFactory {
    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "value" {
            self.value = value;
        } else if name == "freq" {
            self.frequency_hz = value;
        }
    }

    fn get_parameters(&self) -> HashMap<String, f64> {
        HashMap::from([(String::from("value"), self.value), (String::from("freq"), self.frequency_hz)])
    }

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour> {
        Box::new(bipoles::SinusoidalVoltageSource::new(self.value, self.frequency_hz))
    }
}

struct CurrentSourceFactory {
    value: f64
}

impl BipoleFactory for CurrentSourceFactory {
    fn set_parameter(&mut self, name: &str, value: f64) {
        if name == "value" {
            self.value = value;
        }
    }

    fn get_parameters(&self) -> HashMap<String, f64> {
        HashMap::from([(String::from("value"), self.value)])
    }

    fn make(&self) -> Box<dyn bipoles::BipoleBehaviour> {
        Box::new(bipoles::CurrentSource::new(self.value))
    }
}



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
    kind: String
}

impl BipoleToPlace {
    fn new(kind: String) -> BipoleToPlace {
        let (x, y) = mouse_position();
        BipoleToPlace { 
            size: vec2(50.0, 20.0), 
            rotation: BipoleRotation::AnodeUp, 
            center_position: vec2(x, y),
            kind: kind}
    }
}

fn get_anode_position(size: Vec2, center_position: Vec2, rotation: BipoleRotation) -> Vec2{
    let anode_pos_rel = vec2(size.x/2.0, 0.0);
    let matrix = BipoleRotation::get_matrix(rotation.get_angle());

    matrix * anode_pos_rel + center_position
}

fn get_catode_position(size: Vec2, center_position: Vec2, rotation: BipoleRotation) -> Vec2{
    let catode_pos_rel = vec2(-size.x/2.0, 0.0);
    let matrix = BipoleRotation::get_matrix(rotation.get_angle());

    matrix * catode_pos_rel + center_position
}

struct PlacedBipole {
    name: String,
    anode_node_id: usize,
    catode_node_id: usize,
    size: Vec2,
    center_position: Vec2,
    rotation: BipoleRotation,
    factory: Box<dyn BipoleFactory>
}

impl PlacedBipole {
    fn new(name: String, bipole: &BipoleToPlace, anode_id: usize, catode_id: usize) -> PlacedBipole {
        let factory: Box<dyn BipoleFactory>;
        match bipole.kind.as_str() {
            "resistor" => {
                factory = Box::new(ResistorFactory {resistance: 10.0})
            }
            "voltage source" => {
                factory = Box::new(VoltageSourceFactory {value: 10.0})
            }
            "capacitor" => {
                factory = Box::new(CapacitorFactory {capacitance: 2e-5})
            }
            "inductor" => {
                factory = Box::new(InductorFactory {induttance: 2e-5})
            }
            "current source" => {
                factory = Box::new(CurrentSourceFactory {value: 1e-3})
            }
            "diode" => {
                factory = Box::new(DiodeFactory {current_s: 1.0e-15, voltage_vt: 26e-3})
            }
            "sinusoidal" => {
                factory = Box::new(SinusoidalVoltageSourceFactory {value: 10.0, frequency_hz: 1.0})
            }
            _ => {
                factory = Box::new(ResistorFactory {resistance: 10.0})
            }
        }

        PlacedBipole { 
            name: name, 
            anode_node_id: anode_id, 
            catode_node_id: catode_id,
            size: bipole.size,
            center_position: bipole.center_position,
            rotation: bipole.rotation.clone(),
            factory: factory
            }
    }
}

fn draw_bipole(size: Vec2, center_position: Vec2, rotation: BipoleRotation) {
    let rect = rotation.get_rect(size, center_position);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, GREEN);
    
}

trait Mode {
    fn draw(&mut self) {}

    fn update(&mut self, event: ClickEvent, info: UiInfo) -> Option<Command>;
}


enum Command  {
    PlaceBipole(BipoleToPlace),
    PlaceWire{node1_id: usize, node2_id: usize, node2_pos: Vec2, is_new: bool},
    ChangeMode(Box<dyn Mode>),
    ChangeName{old_name: String, new_name: String},
    DeleteBipole {name:String},
    DeleteWire {id: usize},
    ChangeParameters{name: String, parameters: HashMap<String, f64>},
    RunSimulation{sim_time: f64, t_step: f64},
    SetPlotInfo(Option<PlotInfo>),
    SetGround(usize)
}
struct DeleteMode {
}
impl DeleteMode {
    fn new() -> DeleteMode {
        DeleteMode {  }
    }
}

impl Mode for DeleteMode {
    fn draw(&mut self) {
        
    }

    fn update(&mut self, event: ClickEvent, info: UiInfo) -> Option<Command> {
        if let ClickEvent::ToolbarClicked(_) = event {
            let mut mode = ClickMode::new();
            return mode.update(event, info);
        }

        match event {
            ClickEvent::BipoleClicked { name, parameters: _ } => {
                return Some(Command::DeleteBipole { name });
            }
            ClickEvent::WireClicked { wire_id : id } => {
                return Some(Command::DeleteWire { id } );
            }
            _ => {return  None;}

        }
    }
}

struct SetGroundMode {
}
impl SetGroundMode {
    fn new() -> SetGroundMode {
        SetGroundMode {  }
    }
}

impl Mode for SetGroundMode {
    fn draw(&mut self) {
        
    }

    fn update(&mut self, event: ClickEvent, info: UiInfo) -> Option<Command> {
        if let ClickEvent::ToolbarClicked(_) = event {
            let mut mode = ClickMode::new();
            return mode.update(event, info);
        }

        match event {
            ClickEvent::NodeClicked { node_id : id } => {
                return Some(Command::SetGround(id) );
            }
            _ => {return  None;}

        }
    }
}


struct MeasureMode {
}

impl MeasureMode {
    fn new() -> MeasureMode {
        MeasureMode {  }
    }
}

impl Mode for MeasureMode {
    fn draw(&mut self) {
        
    }

    fn update(&mut self, event: ClickEvent, info: UiInfo) -> Option<Command> {
        if let ClickEvent::ToolbarClicked(_) = event {
            let mut mode = ClickMode::new();
            return mode.update(event, info);
        }

        match event {
            ClickEvent::BipoleClicked { name, parameters: _ } => {
                let info = PlotInfo::Current(name);
                return Some(Command::SetPlotInfo(Some(info)));
            }
            ClickEvent::NodeClicked { node_id } => {
                let info = PlotInfo::NodeVolatge(node_id);
                return Some(Command::SetPlotInfo(Some(info)));
            }
            ClickEvent::CanvasClicked => {
                return Some(Command::SetPlotInfo(None));
            }
            _ => {return  None;}

        }
    }
}

struct ClickMode {
    clicked: bool,
    pos: Vec2,
    name: Option<String>,
    parameters: Option<HashMap<String, f64>>,
    current_input: Option<HashMap<String, String>>,
    changed: bool
}

impl ClickMode {
    fn new() -> ClickMode {
        ClickMode { clicked: false, 
            pos: vec2(0.0, 0.0), name: None, parameters: None, current_input: None, changed: false}
    }
}

impl Mode for ClickMode {

    fn draw(&mut self) {
        if self.clicked {
            widgets::Window::new(hash!(), self.pos, vec2(200., 200.))
                .label("Parameters")
                .titlebar(true)
                .ui(&mut *root_ui(), |ui| {
                    for (parameter, _) in self.parameters.as_mut().unwrap() {
                        let current_input = self.current_input.as_mut().unwrap();
                        let current_value = current_input.get_mut(parameter).unwrap();
                        ui.input_text(hash!(), &parameter, 
                            current_value);
                    }

                    if ui.button(vec2(0.0, 20.0), "Ok") {
                        self.clicked = false;
                        self.changed = true;
                        for (parameter, current_value) in self.current_input.as_ref().unwrap() {
                            self.parameters.as_mut().unwrap().insert(parameter.clone(), 
                                    current_value.parse::<f64>().unwrap());
                            
                        }
                    }

                });
        }
        
    }

    fn update(&mut self, event: ClickEvent, _info: UiInfo) -> Option<Command> {
        if is_mouse_button_pressed(MouseButton::Right) {
            self.clicked = false;
            return None;
        }

        if self.changed {
            self.changed = false;
            return Some(Command::ChangeParameters { name: self.name.take().unwrap(), 
                                                parameters: self.parameters.take().unwrap() });
        }

        match event {
            ClickEvent::ToolbarClicked(ToolBarEvent::ArrowClicked)  => {
                Some(Command::ChangeMode(Box::new(ClickMode::new())))
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::PlaceClicked)  => {
                Some(Command::ChangeMode(Box::new(PlaceMode::new())))
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::WireClicked)  => {
                Some(Command::ChangeMode(Box::new(WireMode::new())))
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::RunClicked)  => {
                Some(Command::ChangeMode(Box::new(RunMode::new())))
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::MeasureClicked)  => {
                Some(Command::ChangeMode(Box::new(MeasureMode::new())))
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::DeleteClicked)  => {
                Some(Command::ChangeMode(Box::new(DeleteMode::new())))
            }
            ClickEvent::ToolbarClicked(ToolBarEvent::SetGroundClicked)  => {
                Some(Command::ChangeMode(Box::new(SetGroundMode::new())))
            }
            ClickEvent::BipoleClicked { name, parameters } => {
                if self.clicked {
                    return  None;
                }
                let (x, y) = mouse_position();
                self.clicked = true;
                self.name = Some(name);
                let current_input: HashMap<String, String> = HashMap::from_iter(parameters.iter()
                                .map(|(key, val)| {(key.clone(),String::from(val.to_string()))}));
                self.parameters = Some(parameters);
                
                self.current_input = Some(current_input);
                self.pos = vec2(x, y);
                None
            }
            _ => {None}



        }
    }
}


struct RunMode {
    clicked: bool,
    simulation_over: bool,
    simulation_time: f64,
    time_step: f64,
    sim_time_input: String,
    t_step_input: String
}

impl RunMode {
    fn new() -> RunMode {
        RunMode { clicked: false, simulation_over: false, simulation_time: 0.0, time_step: 0.0,
            sim_time_input: String::new(), t_step_input: String::new() }
    }
}

impl Mode for RunMode {
    fn draw(&mut self) {
        widgets::Window::new(hash!(), vec2(100., 100.), vec2(200., 200.))
                .label("Simulation")
                .titlebar(true)
                .ui(&mut *root_ui(), |ui| {
                    ui.input_text(hash!(), "simulation time", 
                        &mut self.sim_time_input);
                    ui.input_text(hash!(), "time step", 
                        &mut self.t_step_input);

                    if ui.button(vec2(0.0, 50.0), "Ok") {
                        self.clicked = true;
                        self.simulation_time = self.sim_time_input.parse::<f64>().unwrap();
                        self.time_step = self.t_step_input.parse::<f64>().unwrap();
                    } 
                });
    }

    fn update(&mut self, _event: ClickEvent, _info: UiInfo) -> Option<Command> {
        if self.clicked {
            self.clicked = false;
            self.simulation_over = true;

            return Some(Command::RunSimulation{sim_time: self.simulation_time, t_step: self.time_step});
        }
        if self.simulation_over {
            return Some(Command::ChangeMode(Box::new(ClickMode::new())));
        }
        None
    }
}


struct PlaceMode {
    bipole: BipoleToPlace,
    selected: bool,
    components: Vec<String>,
    window_rect: Rect
}

impl PlaceMode {
    fn new() -> PlaceMode {
        PlaceMode {
            bipole: BipoleToPlace::new(String::from("resistor")),
            components: vec![
                String::from("resistor"), 
                String::from("voltage source"), 
                String::from("current source"), 
                String::from("capacitor"), 
                String::from("inductor"),
                String::from("diode"),
                String::from("sinusoidal")],
            selected: false,
            window_rect: Rect::new(10.0, 10.0, 100.0, 400.0)}
        }

    fn is_inside_window(&self, pos: Vec2)-> bool {
        return self.window_rect.contains(pos);
    }
}

impl Mode for PlaceMode {
    fn draw(&mut self) {
        let topleft = vec2(self.window_rect.top(), self.window_rect.left());
        widgets::Window::new(hash!(), topleft, self.window_rect.size())
            .label("Components")
            .titlebar(true)
            .ui(&mut *root_ui(), |ui| {
                for component in &self.components {
                    Group::new(hash!(component), vec2(300., 80.))
                    .ui(ui, |ui| {
                        Group::new(hash!(component, "lab"), vec2(120., 70.))
                            .ui(ui, |ui| {
                                if ui.button(Vec2::new(10., 10.), component.as_str()) {
                                    self.selected = true;
                                    self.bipole = BipoleToPlace::new(component.clone());
                                    
                                }
                            
                            });
                        
                    });
                }
            });
        if self.selected {
            let anode_pos = get_anode_position(self.bipole.size, self.bipole.center_position, self.bipole.rotation);
            draw_bipole(self.bipole.size, self.bipole.center_position, self.bipole.rotation);
            draw_text("+", anode_pos.x +10.0, anode_pos.y, 15.0, BLACK);

        }
    }

    fn update(&mut self, event: ClickEvent, info: UiInfo) -> Option<Command>{
        let (x, y) = mouse_position();
        self.bipole.center_position = vec2(x, y);

        if let ClickEvent::ToolbarClicked(_) = event {
            let mut mode = ClickMode::new();
            return mode.update(event, info);
        }

        if let ClickEvent::CanvasClicked = event {
            if self.is_inside_window(vec2(x, y)) {
                return None;
            }
            return Some(Command::PlaceBipole(BipoleToPlace { 
                size: self.bipole.size, 
                center_position: self.bipole.center_position, 
                rotation: self.bipole.rotation,
                kind: self.bipole.kind.clone() }));
        }

        if is_mouse_button_down(MouseButton::Right) {
            return Some(Command::ChangeMode(Box::new(ClickMode::new())));
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

    fn get_pos(&self, pos2: Vec2) -> Vec2 {
        if (pos2 - self.current_wire_pos1).abs_diff_eq(vec2(0.0, 0.0), 0.01) {
            return pos2;
        }
        let vec = pos2 - self.current_wire_pos1;
        let cos_x = vec.x/vec.dot(vec).sqrt();
        let val = (2.0).sqrt()/2.0;
        if cos_x.abs() > val  {
            vec2(pos2.x, self.current_wire_pos1.y)
        } else  {
            vec2(self.current_wire_pos1.x, pos2.y)
        }  
    }
}

impl Mode for WireMode {
    fn draw(&mut self) {
        if self.drawing {
            let (x, y) = mouse_position();
            let Vec2 {x, y} = self.get_pos(vec2(x, y));
            let Vec2 {x: x1, y: y1} = self.current_wire_pos1;

            draw_line(x1, y1, x, y, 1.0, BLACK);
        }
    }

    fn update(&mut self, event: ClickEvent, info: UiInfo) -> Option<Command> {

        let (x, y) = mouse_position();
        let Vec2 {x, y} = self.get_pos(vec2(x, y));

        if let ClickEvent::ToolbarClicked(_) = event {
            let mut mode = ClickMode::new();
            return mode.update(event, info);
        }

        if self.drawing {
            if let ClickEvent::NodeClicked { node_id } = event {
                if is_mouse_button_down(MouseButton::Left){
                    self.drawing = false;
                    self.current_wire_pos2 = vec2(x, y);
                    self.current_wire_node2_id = node_id;
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
                    let (x, y) = mouse_position();
                    self.drawing = true;
                    self.current_wire_pos1 = vec2(x, y);
                    self.current_wire_node1_id = node_id;
                } 
            }
        }

        if is_mouse_button_down(MouseButton::Right) {
            self.drawing = false;
        }
        None
        
    }
}

enum PlotInfo {
    Current(String),
    NodeVolatge(usize)
}

struct UiData {
    nodes: HashMap<usize, Node>,
    current_node_id: usize,
    wires: HashMap<usize, Wire>,
    current_wire_id: usize,
    placed_bipoles: HashMap<String, PlacedBipole>,
    current_bipole_id: usize,
    mode: Box<dyn Mode>,
    simulation_output: Option<bipoles::SimulationOutput>,
    plot_info: Option<PlotInfo>,
    ground_id: Option<usize>
}

impl UiData {

    pub fn new() -> UiData {

        let mode = ClickMode::new();

        UiData { nodes: HashMap::new(), 
            current_node_id: 0, 
            wires: HashMap::new(), 
            current_wire_id: 0, 
            placed_bipoles: HashMap::new(),
            current_bipole_id: 0,
            mode: Box::new(mode),
            simulation_output: None,
            plot_info: None, 
            ground_id: None
        }
    }

    pub fn add_node(&mut self, pos: Vec2) {
        self.current_node_id += 1;
        self.nodes.insert(self.current_node_id, Node { position: pos, computed_id: self.current_node_id });
    }

    pub fn add_wire(&mut self, node1_id: usize, node2_id: usize) {
        self.current_wire_id += 1;
        self.wires.insert(self.current_wire_id, Wire { 
            node1_pos: self.nodes.get(&node1_id).unwrap().position, 
            node2_pos: self.nodes.get(&node2_id).unwrap().position, 
            node1_id, node2_id});
    }

    pub fn add_bipole(&mut self, bipole: &BipoleToPlace) {
        self.add_node(get_anode_position(bipole.size, bipole.center_position, bipole.rotation));
        let anode_id = self.current_node_id;

        self.add_node(get_catode_position(bipole.size, bipole.center_position, bipole.rotation));
        let catode_id = self.current_node_id;

        self.current_bipole_id += 1;
        let name = String::from(&bipole.kind[0..1]) + &self.current_bipole_id.to_string();
        self.placed_bipoles.insert(name.clone(), 
            PlacedBipole::new(name, bipole, anode_id, catode_id));
    }

    pub fn run(&mut self, sim_time: f64, t_step: f64) {

        for (_, wire) in &self.wires {
            let id1 = self.nodes.get(&wire.node1_id).unwrap().computed_id;
            let id2 = self.nodes.get(&wire.node2_id).unwrap().computed_id;

            if id1 == id2 {
                continue;
            }

            for (_, node) in &mut self.nodes {
                if node.computed_id == id1 {
                    node.computed_id = id2;
                }
            }

        }

        let mut current_mapping = HashMap::new();
        let mut current_index = 0;

        for (_, node) in &mut self.nodes {
            if current_mapping.contains_key(&node.computed_id) {
                node.computed_id = *current_mapping.get(&node.computed_id).unwrap();
            } else {
                current_mapping.insert(node.computed_id, current_index);
                node.computed_id = current_index;
                current_index += 1;
            }

        }

        let ground_id;
        if let Some(id) = self.ground_id {
            ground_id = self.nodes.get(&id).unwrap().computed_id;
        } else {
            ground_id = 0;
        }

        let mut circ = bipoles::Circuit::new(ground_id);


        for (id, bipole) in &self.placed_bipoles {
            let anode_id = self.nodes.get(&bipole.anode_node_id).unwrap().computed_id;
            let catode_id = self.nodes.get(&bipole.catode_node_id).unwrap().computed_id;

            println!("{id}: {anode_id}, {catode_id}");
            circ.add_bipole(bipole.factory.make(), 
                anode_id, catode_id, 
                bipole.name.clone())
        }

        self.simulation_output =  Some(circ.simulate(sim_time, t_step));
    }

    pub fn is_colliding_node(&self, pos: Vec2) -> Option<usize> {
        for (id, node) in &self.nodes {
            if pos.distance(node.position) < 5.0 {
                return Some(*id)
            }
        }
        None
    }

    pub fn is_colliding_bipole(&self, pos: Vec2) -> Option<&str> {
        for (name, bipole) in &self.placed_bipoles {
            let rect = bipole.rotation.get_rect(bipole.size, bipole.center_position);
            if rect.contains(pos) {
                return Some(name);
            }
        }
        None
    }

    pub fn is_colliding_wire(&self, pos: Vec2) -> Option<usize> {
        for (id, wire) in &self.wires {
            let dst_sum = pos.distance(wire.node1_pos) +pos.distance(wire.node2_pos);
            let pt_dst = wire.node1_pos.distance(wire.node2_pos);
            if (pt_dst - dst_sum).abs() < 2.0 {
                return  Some(*id);
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

            if let Some(name) = self.is_colliding_bipole(vec2(x, y)) {
                let bipole = self.placed_bipoles.get(name).unwrap();
                return  ClickEvent::BipoleClicked { 
                    name: String::from(name), 
                    parameters: bipole.factory.get_parameters() };
            }

            if let Some(id) = self.is_colliding_wire(vec2(x, y)){
                return ClickEvent::WireClicked { wire_id: id };
            }
            return ClickEvent::CanvasClicked;
        } else {
            return ClickEvent::ToolbarClicked(event);
        }
        
    }

    pub fn update(&mut self, event: ToolBarEvent){
        
        let click_event = self.generate_click_event(event);
        let info = UiInfo {current_node_id: self.current_node_id};

        if let  Some(command) = self.mode.update(click_event, info) {
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
                Command::ChangeName { old_name, new_name } => {
                    let bipole = self.placed_bipoles.remove(&old_name).unwrap();
                    self.placed_bipoles.insert(new_name, bipole);

                }
                Command::ChangeParameters { name, parameters } => {
                    let bipole = self.placed_bipoles.get_mut(&name).unwrap();
                    for (par_name, value) in &parameters {
                        bipole.factory.set_parameter(&par_name, *value);
                    }
                }
                Command::RunSimulation { sim_time, t_step } => {
                    self.run(sim_time, t_step);
                }
                Command::SetPlotInfo(info) => {
                    self.plot_info = info;
                }
                Command::DeleteBipole { name } => {
                    let bipole = self.placed_bipoles.get(&name).unwrap();
                    self.nodes.remove(&bipole.anode_node_id);
                    self.nodes.remove(&bipole.catode_node_id);
                    self.placed_bipoles.remove(&name);
                }
                Command::DeleteWire { id } => {
                    let wire = self.wires.get(&id).unwrap();
                    self.nodes.remove(&wire.node1_id);
                    self.nodes.remove(&wire.node2_id);
                    self.wires.remove(&id);
                }
                Command::SetGround(id) => {
                    self.ground_id = Some(id);
                }
            }
        }
    }

    fn plot(&self) {
        if let Some(_) = self.simulation_output{
            let values: &Vector<f64>;
            match &self.plot_info {
                Some(PlotInfo::Current(id)) => {
                    values = self.simulation_output.as_ref().unwrap().currents.get(id).unwrap();
                }
                Some(PlotInfo::NodeVolatge(id)) => {
                    let computed_id = self.nodes.get(id).unwrap().computed_id;
                    values = self.simulation_output.as_ref().unwrap().node_voltages.get(&computed_id).unwrap();
                }
                None => {return;}
            }
            let rect = Rect::new(200.0, 200.0, 700.0, 500.0);
            let points = PlotIterator::new(&values, rect.w, rect.h);
            
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, GRAY);
            let (max, min) = points.get_max_min();

            for (point1, point2) in points {
                draw_line(point1.x + rect.left(), rect.bottom() - point1.y as f32, 
                            point2.x +rect.left(), rect.bottom() - point2.y as f32, 
                            1.0, BLACK);
            }
            draw_text(&max.to_string(), rect.left(), rect.top(), 15.0, BLACK);
            draw_text(&min.to_string(), rect.left(), rect.bottom(), 15.0, BLACK);
            


        }
        
    }

    pub fn draw(&mut self) {
        
        self.mode.draw();

        for (name, bipole) in &self.placed_bipoles {

            let (x, y) = (bipole.center_position.x, bipole.center_position.y);
            let anode_pos = get_anode_position(bipole.size, bipole.center_position, bipole.rotation);

            draw_bipole(bipole.size, bipole.center_position, bipole.rotation);
            draw_text(name, x, y, 15.0, BLACK);
            draw_text("+", anode_pos.x +10.0, anode_pos.y, 15.0, BLACK);
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
    RunClicked,
    MeasureClicked,
    DeleteClicked,
    SetGroundClicked,
    NoneClicked

}

enum ClickEvent {
    ToolbarClicked(ToolBarEvent),
    NodeClicked {node_id : usize},
    BipoleClicked {name: String, parameters: HashMap<String, f64>},
    WireClicked {wire_id: usize},
    CanvasClicked,
    NoneClicked
}

struct UiInfo {
    current_node_id: usize
}

#[macroquad::main("UI Circuit sim")]
async fn main() {

    let texture: Texture2D = load_texture("assets/resistor.png").await.unwrap();
    let mut uidata = UiData::new();

    loop {
        clear_background(WHITE);

        let mut toolbar_event = ToolBarEvent::NoneClicked;

        widgets::Window::new(hash!(), vec2(0., 0.), vec2(1000., 50.))
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

                    if ui.button(vec2(300.0, 0.0), "Run mode") {
                        toolbar_event = ToolBarEvent::RunClicked;
                    }

                    if ui.button(vec2(400.0, 0.0), "Measure mode") {
                        toolbar_event = ToolBarEvent::MeasureClicked;
                    }

                    if ui.button(vec2(500.0, 0.0), "Delete mode") {
                        toolbar_event = ToolBarEvent::DeleteClicked;
                    }

                    if ui.button(vec2(600.0, 0.0), "Set ground") {
                        toolbar_event = ToolBarEvent::SetGroundClicked;
                    }
                

            });

        uidata.update(toolbar_event);
        uidata.draw();
        uidata.plot();
        

        next_frame().await;
    }
}
