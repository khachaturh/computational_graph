use crate::graph::NodeType::{BinaryB2nd, BinaryFn, Param, UnaryFn};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Node {
    inputs: Vec<Rc<RefCell<Node>>>,
    outputs: Vec<Weak<RefCell<Node>>>,
    cache: Option<f32>,
    name: String,
    node_t: NodeType,
}

pub type ShNode = Rc<RefCell<Node>>;

enum NodeType {
    Param,
    UnaryFn(fn(f32) -> f32),
    BinaryFn(fn(f32, f32) -> f32),
    BinaryB2nd(fn(f32, f32) -> f32, f32),
}

pub trait Setter {
    fn set(&self, value: f32);
}

impl Setter for ShNode {
    fn set(&self, value: f32) {
        self.borrow_mut().set(value);
    }
}

pub trait Computer {
    fn compute(&self) -> f32;
}

impl Computer for ShNode {
    fn compute(&self) -> f32 {
        self.borrow_mut().compute()
    }
}

impl Node {
    fn new(name: &str, node_t: NodeType, inputs: Vec<ShNode>) -> ShNode {
        let node = Rc::new(RefCell::new(Node {
            inputs,
            outputs: Vec::new(),
            cache: None,
            name: name.to_string(),
            node_t,
        }));

        for input in node.borrow().inputs.iter() {
            input.borrow_mut().outputs.push(Rc::downgrade(&node));
        }

        node
    }

    fn set(&mut self, value: f32) {
        if let Param = self.node_t {
            self.invalidate();
            self.cache = Some(value);
        }
    }

    fn compute(&mut self) -> f32 {
        if let Some(cache) = self.cache {
            cache
        } else {
            let new_value = match self.node_t {
                // notes: call unwrap_or_else instead of .expect for lazy string format creation
                Param => self
                    .cache
                    .unwrap_or_else(|| panic!("Please set value for input {}", self.name)),
                UnaryFn(f) => f(self.inputs[0].borrow_mut().compute()),
                BinaryFn(f) => {
                    let x1 = self.inputs[0].borrow_mut().compute();
                    let x2 = self.inputs[1].borrow_mut().compute();
                    f(x1, x2)
                }
                BinaryB2nd(f, x) => f(self.inputs[0].borrow_mut().compute(), x),
            };
            *self.cache.insert(new_value)
        }
    }

    // recursively invalidate all output nodes
    fn invalidate(&mut self) {
        self.cache = None;

        for node in self.outputs.iter_mut() {
            node.upgrade().map(|n| n.borrow_mut().invalidate());
        }
    }
}

pub fn create_input(name: &str) -> ShNode {
    Node::new(name, Param, Vec::new())
}

pub fn add(input1: ShNode, input2: ShNode) -> ShNode {
    Node::new("add", BinaryFn(|x, y| x + y), vec![input1, input2])
}

pub fn mul(input1: ShNode, input2: ShNode) -> ShNode {
    Node::new("mul", BinaryFn(|x, y| x * y), vec![input1, input2])
}

pub fn pow_f32(input: ShNode, n: f32) -> ShNode {
    Node::new("pow", BinaryB2nd(|x, y| x.powf(y), n), vec![input])
}

pub fn sin(input: ShNode) -> ShNode {
    Node::new("sin", UnaryFn(|x| x.sin()), vec![input])
}


#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2};
    use std::rc::Rc;
    use crate::{add, Computer, create_input, mul, pow_f32, Setter, sin};

    fn round(x: f32, precision: u32) -> f32 {
        let m = 10i32.pow(precision) as f32;
        (x * m).round() / m
    }

    #[test]
    fn simple() {
        let x1 = create_input("x1");
        x1.set(2.0);
        assert_eq!(x1.compute(), 2.0);

        // test for the same cloned node
        let graph = add(x1.clone(), x1.clone());
        x1.set(5.0);
        let result =  graph.compute();
        let result = round(result, 5);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn fib() {
        let x1 = create_input("x1");
        let x2 = create_input("x2");

        x1.set(1.0);
        x2.set(1.0);

        let mut a1 = x1.clone();
        let mut a2 = x1.clone();

        for _ in 0..6 {
            let tmp = a2.clone();
            a2 = add(a1, a2);
            a1 = tmp;
        }

        let result = a2.compute();
        let result = round(result, 5);
        assert_eq!(result, 21.0);

        x1.set(-1.0);
        x2.set(-1.0);

        let result = a2.compute();
        let result = round(result, 5);
        assert_eq!(result, -21.0);
    }

    #[test]
    fn pow_mul() {
        // z = (x ^ 6) * (x ^ 3) * x = x ^ 10
        let x = create_input("x");
        x.set(2.0);

        let graph1 = mul(mul(pow_f32(x.clone(), 6.0),
                            pow_f32(x.clone(), 3.0)), x.clone());
        let graph2 = pow_f32(x.clone(), 10.0);

        let result1 = graph1.compute();
        let result1 = round(result1, 5);
        let result2 = graph2.compute();
        let result2 = round(result2, 5);

        assert_eq!(result1, 1024.0);
        assert_eq!(result1, result2);
    }

    #[test]
    fn sin_pow() {
        let x = create_input("x");
        let graph = pow_f32(sin(x.clone()), 2.0);
        x.set(FRAC_PI_2);
        let result =  graph.compute();
        let result = round(result, 5);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn cycle_ref() {
        // check on cyclic refs
        let weak = {
            let x = create_input("x");
            x.set(0.0);
            let graph = mul(add(x.clone(), x.clone()), add(x.clone(), x.clone()));
            x.set(1.0);
            let result =  graph.compute();
            let _result = round(result, 5);

            let weak = Rc::downgrade(&x);

            // there are 5 strong refs: x and 4 clones
            assert_eq!(weak.strong_count(), 5);
            weak
        };

        // check if node fully dropped
        assert_eq!(weak.strong_count(), 0);
    }
}


