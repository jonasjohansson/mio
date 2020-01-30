"use strict";
const Store = require("electron-store");

module.exports = new Store({
  defaults: {
    lastWindowState: {
      x: 0,
      y: 0,
      width: 200,
      height: 400
    },
    baudrates: [9600, 115200],
    interval: 100,
    port: 8080,
    remoteServer: "",
    keys: ["mouse", "backspace", "delete", "enter", "tab", "escape", "up", "down", "left", "right", "home", "end", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12", "command", "alt", "control", "shift", "right_shift", "space", "printscreen", "insert", "audio_mute", "audio_vol_down", "audio_vol_up", "audio_play", "audio_stop", "audio_prev", "audio_next", "lights_mon_up", "lights_mon_down", "lights_kbd_toggle", "lights_kbd_up", "lights_kbd_down", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
    mods: ["none", "alt", "command", "control", "shift"]
  }
});
