use macroquad::prelude::{Vec2, vec2};
use mathru::algebra::linear::Vector;
use std::iter::Iterator;

pub struct PlotIterator<'a> {
    current_index: usize,
    values: &'a Vector<f64>,
    num_vals: usize,
    abs_max: f64,
    width: f32,
    height: f32,
    increment: f32,
    max: f64,
    min: f64
}

impl <'a> PlotIterator<'a> {
    pub fn new(values: &'a Vector<f64>, width: f32, height: f32) -> PlotIterator<'a> {
        let mut max_abs =  0.0;
        let mut max = 1.0e-75;
        let mut min = -1.0e-75;
        let mut count = 0;
        for value in values.iter() {
            if value.abs() > max_abs {
                max_abs = value.abs();
            }
            if value > &max {
                max = *value;
            }
            if value < &min {
                min = *value;
            }
            count += 1;
        }
        PlotIterator {
            current_index: 0,
            values: values,
            num_vals: count,
            abs_max: max_abs,
            width: width,
            height: height,
            increment: width/(count as f32),
            max: max,
            min: min
        }
    }

    pub fn get_max_min(&self) -> (f64, f64) {
        (self.max, self.min)
    }

    pub fn get_nvals(&self) -> usize {
        self.num_vals
    }

    fn normalize(&self, value: f64) -> f64 {

        let value_norm = value - self.min;
        (value_norm/(self.max - self.min)) *(self.height as f64)
    }
}

impl <'a> Iterator for PlotIterator<'a> {
    type Item =  (Vec2, Vec2);

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.current_index +1 == self.num_vals {
            return None;
        }
        let past_value = self.normalize(self.values[self.current_index]);
        let current_value = self.normalize(self.values[self.current_index+1]);
        let past_x = self.current_index as f32 * self.increment;
        let current_x = ((self.current_index+1) as f32) * self.increment;

        self.current_index += 1;
        
        return Some((vec2(past_x, past_value as f32), vec2(current_x, current_value as f32)));
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {

        let mut values = Vector::one(10) *4.5;
        values.set_slice(&(Vector::one(5)*2.5), 4);

        let points = PlotIterator::new(&values, 300.0, 100.0);
        let max = points.abs_max;
        let incr = points.increment;
        println!("{incr}");
        let n = points.num_vals;
        println!("{n}");
        for (point1, point2) in points {
            println!("{point1}");
            println!("{point2}");
        }
        
        
        println!("{max}");

    }
}