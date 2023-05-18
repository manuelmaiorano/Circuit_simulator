use std::collections::{HashMap, HashSet};
use mathru::algebra::linear::{matrix::{Solve},Matrix, Vector};
use std::f64::consts;


pub enum Model {
    ConduttanceCurrentSource{conduttance: f64, current: f64},
    VoltageSource(f64)
}

pub trait BipoleBehaviour {

    fn linear_companion(&self, timestep_sec: f64, current_time_sec: f64) -> Model;

    fn is_dynamic(&self) -> bool {false}

    fn is_nonlinear(&self) -> bool {false}

    fn update_state(&mut self, _anode_tension: f64,_catode_tensionn: f64, _timestep_sec: f64) {}

    fn update_operating_point(&mut self, _anode_tension: f64, _catode_tension: f64, _current: f64) {}
}



pub struct Resistor {
    resistance: f64
}

impl BipoleBehaviour for Resistor {
    fn linear_companion(&self, _timestep_sec: f64, _current_time_sec: f64) -> Model {
        Model::ConduttanceCurrentSource{
            conduttance: 1.0/self.resistance, 
            current: 0.0
        
        }
    }
}


pub struct CurrentSource {
    value: f64
}

impl BipoleBehaviour for CurrentSource {
    fn linear_companion(&self, _timestep_sec: f64, _current_time_sec: f64) -> Model {
        Model::ConduttanceCurrentSource{
            conduttance: 0.0, 
            current: self.value
        
        }
    }
}


pub struct VoltageSource {
    value: f64
}

impl BipoleBehaviour for VoltageSource {
    fn linear_companion(&self, _timestep_sec: f64, _current_time_sec: f64) -> Model {
        Model::VoltageSource(self.value)
    }
}


pub struct SinusoidalVoltageSource {
    value: f64,
    frequency_hz: f64,

}

impl BipoleBehaviour for SinusoidalVoltageSource {
    fn linear_companion(&self, _timestep_sec: f64, _current_time_sec: f64) -> Model {
        Model::VoltageSource(self.value * (self.frequency_hz* 2.0 *consts::PI).sin() )
    }
}


pub struct Capacitor {
    capacitance: f64,
    current_voltage: f64
}

impl BipoleBehaviour for Capacitor {
    fn is_dynamic(&self) -> bool {
        true
    }

    fn linear_companion(&self, timestep_sec: f64, _current_time_sec: f64) -> Model {
        Model::ConduttanceCurrentSource{
            conduttance: self.capacitance/timestep_sec, 
            current: - self.current_voltage * self.capacitance/timestep_sec
        
        }
    }

    fn update_state(&mut self, anode_tension: f64, catode_tension: f64, _timestep_sec: f64) {
        self.current_voltage = anode_tension - catode_tension;
    }
}


pub struct Inductor {
    induttance: f64,
    current_i: f64
}

impl BipoleBehaviour for Inductor {
    fn is_dynamic(&self) -> bool {
        true
    }

    fn linear_companion(&self, timestep_sec: f64, _current_time_sec: f64) -> Model {
        Model::ConduttanceCurrentSource{
            conduttance: timestep_sec/self.induttance, 
            current: - self.current_i
        
        }
    }

    fn update_state(&mut self, anode_tension: f64, catode_tension: f64, timestep_sec: f64) {
        
        let equivalent_conduttance = timestep_sec/self.induttance;
        self.current_i = (anode_tension  - catode_tension)*equivalent_conduttance + self.current_i;
    }
}


pub struct Diode {
    current_s: f64,
    voltage_vt: f64,
    current_i: f64,
    current_v: f64
}

impl BipoleBehaviour for Diode {

    fn is_nonlinear(&self) -> bool {
        true
    }

    fn linear_companion(&self, _timestep_sec: f64, _current_time_sec: f64) -> Model{
        let equivalent_conduttance = self.current_s/self.voltage_vt * (self.current_v/self.voltage_vt).exp();
        Model::ConduttanceCurrentSource{
            conduttance: equivalent_conduttance, 
            current: self.current_i - equivalent_conduttance * self.current_v
        }
    }
    
    fn update_operating_point(&mut self, anode_tension: f64, catode_tension: f64, current:f64){
        let equivalent_conduttance = self.current_s/self.voltage_vt * (self.current_v/self.voltage_vt).exp();
        let voltage = anode_tension - catode_tension;
        self.current_i = self.current_i + equivalent_conduttance * (voltage - self.current_v) ;
        self.current_v = voltage;
    }
}

struct Bipole {
    anode_id: usize,
    catode_id: usize,
    behaviour: Box<dyn BipoleBehaviour>
}

pub struct Circuit{

    bipoles: HashMap<String, Bipole>,
    dynamic_bipoles: HashSet<String>,
    nonlinear_bipoles: HashSet<String>,
    ground_id: usize,
    nodes: HashSet<usize>,
    voltage_bipoles: HashSet<String>
}

impl Circuit {
    pub fn new(ground_id: usize) -> Circuit {
        Circuit { bipoles: HashMap::new(), 
            dynamic_bipoles: HashSet::new(), 
            nonlinear_bipoles: HashSet::new(), 
            ground_id: ground_id, 
            nodes: HashSet::new(), 
            voltage_bipoles: HashSet::new() }
    }

    pub fn add_bipole(&mut self, behaviour: impl BipoleBehaviour + 'static, anode_id: usize, catode_id: usize, name: String){
        
        let is_dynamic = behaviour.is_dynamic();
        let is_non_linear = behaviour.is_nonlinear();

        if let Model::VoltageSource(_) = behaviour.linear_companion(1.0, 1.0) {
            self.voltage_bipoles.insert(name.clone());
        }

        let bipole = Bipole {anode_id, catode_id, behaviour: Box::new(behaviour)};
        if is_dynamic {
            self.dynamic_bipoles.insert(name.clone());
        } 
        if is_non_linear {
            self.nonlinear_bipoles.insert(name.clone());
        } 
        self.nodes.insert(anode_id);
        self.nodes.insert(catode_id);


        self.bipoles.insert(name, bipole);


    }

    fn fill(&mut self, timestep_sec: f64, time: f64, 
        voltage_bipole_to_current_idx: &HashMap<String, usize>,
        matrix: &mut Matrix<f64>,
        sources: &mut Vector<f64>)  {
    
        for (bipole_name, bipole) in &self.bipoles {
            let model = bipole.behaviour.linear_companion(timestep_sec, time);
            match model {
                Model::VoltageSource(value) => {
                    let idx = voltage_bipole_to_current_idx.get(bipole_name).unwrap();
                    let idx = *idx;

                    matrix[[bipole.anode_id, idx]] += 1.0;
                    matrix[[bipole.catode_id, idx]] -= 1.0;

                    matrix[[idx, bipole.anode_id]] += 1.0;
                    matrix[[idx, bipole.catode_id]] -= 1.0;
                    sources[idx] = value;


                },
                Model::ConduttanceCurrentSource{ conduttance, current }=> {

                    sources[bipole.anode_id] -= current;
                    sources[bipole.catode_id] += current;

                    matrix[[bipole.anode_id, bipole.catode_id]] -= conduttance;
                    matrix[[bipole.catode_id, bipole.anode_id]] -= conduttance;
                    
                    matrix[[bipole.anode_id, bipole.anode_id]] += conduttance;
                    matrix[[bipole.catode_id, bipole.catode_id]] += conduttance;

                }
            }


        }

    }

    fn clear(&self, matrix: &mut Matrix<f64>, sources: &mut Vector<f64>) {
        matrix.mut_apply(&|_element| 0.0);
        for data in sources.iter_mut() {
            *data = 0.0;
        }
    }

    fn update_nonlinear_op(&mut self, sol: &Vector<f64>) {
        for non_linear_bipole_name in &self.nonlinear_bipoles {
            let bipole = self.bipoles.get_mut(non_linear_bipole_name).unwrap();

            bipole.behaviour.update_operating_point(sol[bipole.anode_id]
                , sol[bipole.catode_id], 0.0);
        }
    }


    fn solve_nonlinear(&mut self, timestep_sec: f64, time: f64, 
        voltage_bipole_to_current_idx: &HashMap<String, usize>,
        matrix: &mut Matrix<f64>,
        sources: &mut Vector<f64>,
        n_iterations: usize) -> Vector<f64>{

        let mut sol = Vector::zero(matrix.ncols());
        for _ in 0..n_iterations {
            self.clear(matrix, sources);
            self.fill(timestep_sec, time, voltage_bipole_to_current_idx, matrix, sources);
            sol = matrix.solve(sources).unwrap();
            self.update_nonlinear_op(&sol);

        }

        sol

    }

    pub fn simulate(&mut self, simulationtime_sec: f64, timestep_sec: f64) -> SimulationOutput{
        let n_steps: usize = (simulationtime_sec/timestep_sec) as usize;
        let mut out = SimulationOutput{ currents: HashMap::new(), node_voltages: HashMap::new()};

        for (bipole_name, _bipole) in &self.bipoles {
            out.currents.insert(bipole_name.clone(), Vector::zero(n_steps));
        }

        for node in &self.nodes {
            out.node_voltages.insert(*node, Vector::zero(n_steps));
        }

        let number_of_nodes = self.nodes.len();
        let unknowns = self.nodes.len() + self.voltage_bipoles.len();
        let mut matrix: Matrix<f64> = Matrix::zero(unknowns, unknowns);
        let mut sources: Vector<f64> = Vector::zero(unknowns);
        let mut voltage_bipole_to_current_idx: HashMap<String, usize> = HashMap::new();

        for (i, voltage_bipole_name) in self.voltage_bipoles.iter().enumerate() {
            voltage_bipole_to_current_idx.insert(voltage_bipole_name.clone(), number_of_nodes + i);
        }

        for step in 0..n_steps {
            let time = (step as f64) *timestep_sec;
            
            for (bipole_name, bipole) in &self.bipoles {
                let model = bipole.behaviour.linear_companion(timestep_sec, time);
                match model {
                    Model::VoltageSource(value) => {
                        let idx = voltage_bipole_to_current_idx.get(bipole_name).unwrap();
                        let idx = *idx;

                        matrix[[bipole.anode_id, idx]] += 1.0;
                        matrix[[bipole.catode_id, idx]] -= 1.0;

                        matrix[[idx, bipole.anode_id]] += 1.0;
                        matrix[[idx, bipole.catode_id]] -= 1.0;
                        sources[idx] = value;


                    },
                    Model::ConduttanceCurrentSource{ conduttance, current }=> {

                        sources[bipole.anode_id] -= current;
                        sources[bipole.catode_id] += current;

                        matrix[[bipole.anode_id, bipole.catode_id]] -= conduttance;
                        matrix[[bipole.catode_id, bipole.anode_id]] -= conduttance;
                        
                        matrix[[bipole.anode_id, bipole.anode_id]] += conduttance;
                        matrix[[bipole.catode_id, bipole.catode_id]] += conduttance;

                    }
                }


            }

            let sol = matrix.solve(&sources).unwrap();

            for bipole_name in &self.dynamic_bipoles {
                let bipole = self.bipoles.get_mut(bipole_name).unwrap();
                bipole.behaviour.update_state(sol[bipole.anode_id], sol[bipole.catode_id], timestep_sec);
            }

            for (bipole_name, current_vector) in &mut out.currents {
                if let Some(idx) = voltage_bipole_to_current_idx.get(bipole_name) {
                    current_vector[step] = sol[*idx];
                } else {
                    let bipole = self.bipoles.get(bipole_name).unwrap();
                    let model = bipole.behaviour.linear_companion(timestep_sec, time);

                    if let Model::ConduttanceCurrentSource { conduttance, current} = model {
                        current_vector[step] = conduttance *(sol[bipole.anode_id] - sol[bipole.catode_id]) +current;
                    }


                }
                
            }

            for (node_id, voltage_vector) in &mut out.node_voltages {
                voltage_vector[step] = sol[*node_id] - sol[self.ground_id];
            }

            matrix.mut_apply(&|_element| 0.0);
            for data in sources.iter_mut() {
                *data = 0.0;
            }

        }

        out


    }


}

pub struct SimulationOutput {
    pub currents: HashMap<String, Vector<f64>>,
    pub node_voltages: HashMap<usize, Vector<f64>>

}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_solve() {

        let mut circuit = Circuit::new(0);

        circuit.add_bipole(CurrentSource {value:1.0}, 0, 1,String::from("I"));
        circuit.add_bipole(Resistor {resistance:0.1}, 1, 2,String::from("R1"));

        circuit.add_bipole(Resistor {resistance:0.2}, 2, 0,String::from("R2"));
        circuit.add_bipole(Resistor {resistance:0.2}, 2, 0,String::from("R3"));


        let out = circuit.simulate(1.0, 0.5);

        let voltage2 = out.node_voltages.get(&2).unwrap();
        
        println!("{:?}", voltage2);
        assert!((voltage2[0] - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_voltage() {
        let mut circ = Circuit::new(0);

        circ.add_bipole(VoltageSource{value: 10.0}, 1, 0, String::from("V"));
        circ.add_bipole(Resistor{resistance: 10.0}, 1, 2, String::from("R1"));
        circ.add_bipole(Resistor{resistance: 10.0}, 2, 0, String::from("R2"));

        let out = circ.simulate(1.0, 0.5);

        let voltage2 = out.node_voltages.get(&2).unwrap();
        
        println!("{:?}", voltage2);
        assert!((voltage2[0] - 5.0).abs() < 0.01);

    }

    #[test]
    fn test_dynamic() {
        let mut circ = Circuit::new(0);

        // circ.add_voltage(VoltageSource::new(5.0, 1, 0));
        circ.add_bipole(VoltageSource{value: 10.0}, 1, 0, String::from("V"));
        // circ.add_bipole(Resistor::new(5_000.0, 2, 1));
        circ.add_bipole(Resistor{resistance: 5000.0}, 2, 1, String::from("R1"));
        // circ.add_bipole(Capacitor::new(2e-5, 2, 0, 0.0));
        circ.add_bipole(Capacitor{capacitance: 2e-5, current_voltage:0.0}, 2, 0, String::from("C1"));

        // let evolution = circ.solve_dynamic(0.01/2.0, 1.0);
        let out = circ.simulate(1.0, 0.01/2.0);

        let voltage2 = out.node_voltages.get(&2).unwrap();

        assert!((voltage2[voltage2.argmax()] - 10.0).abs() < 0.01);


    }
}
