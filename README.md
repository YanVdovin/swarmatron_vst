# Minimal Audio Plugin in Rust

This is just a starting template for plugins using the nih-plug plugin framework, as I don't like to use the cookiecutter template.

## Build

```sh
cargo xtask bundle swarmer --release
```

## Copy to Plugin Folder

### Linux

Clap:
```sh
cp -r target/bundled/swarmer.clap ~/.clap/
```

VST:
```sh
cp -r target/bundled/swarmer.vst3 ~/.vst3/
```

### Windows

<!-- Clap:
```sh
cp -r target/bundled/swarmer.clap ~/.clap/
``` -->

VST:
```sh
cp -r target/bundled/swarmer.vst3 ~/ProgramFiles/CommonFiles/VST3
```