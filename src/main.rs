use nih_plug::prelude::*;
use swarmer::MyPlugin;

fn main() {
    nih_export_standalone::<MyPlugin>();
}
