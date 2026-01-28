# Minimal Audio Plugin in Rust

This is just a starting template for plugins using the nih-plug plugin framework, as I don't like to use the cookiecutter template.

## Build

```sh
cargo xtask bundle swarmatron_vst --release
```

## Copy to Plugin Folder

### Linux

Clap:
```sh
cp -r target/bundled/swarmatron_vst.clap ~/.clap/
```

VST:
```sh
cp -r target/bundled/swarmatron_vst.vst3 ~/.vst3/
```
