## A simple clipboard manager written in Rust

Easily manage your clipboard entries with a minimal UI.

_Currently under development._

![Clippy Screenshot](./screenshot.png)

## Installation (Linux)

_You need to have `Rust` and `Cargo` installed on your machine to run this tool. Official installation steps [here.](https://www.rust-lang.org/tools/install)_

```bash
git clone https://github.com/Rayanworkout/clippy.git

cd clippy

cargo build --release

sudo cp target/release/clippy /usr/local/bin

nohup clippy &>/dev/null &
```

The application should now be running and listening for your clipboard changes.

#### Disclaimers / Informations

- The GUI and the clipboard listener threads are currently not separated. It means that the app needs to be running in order to listen for clipboard changes.

- If you close the app, your clipboard won't be tracked anymore.

- The history file `clipboard_history.ron` will be located in the folder from which the binary was launched.