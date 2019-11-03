# Homie

Mio simplifies serial communication as a trigger for key presses and MIDI communication. It relies on specific commands (eg. a0 87) being sent by a device (eg. Arduino). The command will be parsed, leaving only the integer.

## Build

### Install Node

https://nodejs.org/en/download/

### Install Electron

https://electronjs.org/releases/stable?version=4

```
npm install electron@4.2.12
```

### Install Mio

https://www.electronforge.io/

```
npm install
npm rebuild --runtime=electron --target=4.2.12 --disturl=https://atom.io/download/atom-shell --abi=69
electron-forge make --platform=mas

```

Move Mio.app to Application folder
