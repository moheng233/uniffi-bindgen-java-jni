use std::sync::Mutex;

// Include the auto-generated scaffolding code from the UDL file
uniffi::include_scaffolding!("simple");

/// Add two unsigned 32-bit integers.
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// Multiply two unsigned 32-bit integers.
pub fn multiply(a: u32, b: u32) -> u32 {
    a * b
}

/// Greet someone by name.
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

/// A data record holding a value and a label.
#[derive(Debug, Clone)]
pub struct MyData {
    pub value: u32,
    pub label: String,
}

/// A simple flat enum representing colors.
#[derive(Debug, Clone)]
pub enum Color {
    Red,
    Green,
    Blue,
}

/// An enum with associated data.
#[derive(Debug, Clone)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Point,
}

/// A calculator object that maintains state.
#[derive(Debug)]
pub struct Calculator {
    value: Mutex<u32>,
}

impl Calculator {
    /// Create a new calculator with an initial value.
    pub fn new(initial: u32) -> Self {
        Self {
            value: Mutex::new(initial),
        }
    }

    /// Add a value to the calculator's current value.
    pub fn add(&self, value: u32) -> u32 {
        let mut v = self.value.lock().unwrap();
        *v += value;
        *v
    }

    /// Subtract a value from the calculator's current value.
    pub fn subtract(&self, value: u32) -> u32 {
        let mut v = self.value.lock().unwrap();
        *v = v.saturating_sub(value);
        *v
    }

    /// Get the current value.
    pub fn get_value(&self) -> u32 {
        *self.value.lock().unwrap()
    }

    /// Process some data and return the result.
    pub fn process_data(&self, input: MyData) -> MyData {
        let mut v = self.value.lock().unwrap();
        *v += input.value;
        MyData {
            value: *v,
            label: format!("processed: {}", input.label),
        }
    }
}

/// A callback interface for reporting calculator events.
pub trait CalculatorListener: Send + Sync {
    fn on_calculation(&self, operation: String, value: u32);
    fn on_reset(&self);
}
