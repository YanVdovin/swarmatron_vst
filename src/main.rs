use nih_plug::prelude::*;
use swarmatron_vst::Swarmer;

fn main() {
    nih_export_standalone::<Swarmer>();
}
