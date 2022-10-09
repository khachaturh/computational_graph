mod graph;
use crate::graph::{add, create_input, mul, pow_f32, sin, Computer, Setter};

// round to decimal digits
fn round(x: f32, precision: u32) -> f32 {
    let m = 10i32.pow(precision) as f32;
    (x * m).round() / m
}

fn main() {
    // x1, x2, x3 are input nodes of the computational graph:
    let x1 = create_input("x1");
    let x2 = create_input("x2");
    let x3 = create_input("x3");
    // graph variable is the output node of the graph:
    let graph = add(
        x1.clone(),
        mul(x2.clone(), sin(add(x2.clone(), pow_f32(x3.clone(), 3f32)))),
    );

    x1.set(1f32);
    x2.set(2f32);
    x3.set(3f32);
    let mut result = graph.compute();
    result = round(result, 5);
    println!("Graph output = {}", result);
    assert_eq!(round(result, 5), -0.32727);

    x1.set(2f32);
    x2.set(3f32);
    x3.set(4f32);
    result = graph.compute();
    result = round(result, 5);
    println!("Graph output = {}", result);
    assert_eq!(round(result, 5), -0.56656);
}
