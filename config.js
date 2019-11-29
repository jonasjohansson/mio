"use strict";
const Store = require("electron-store");

module.exports = new Store({
  defaults: {
    lastWindowState: {
      x: 0,
      y: 0,
      width: 800,
      height: 600
    },
    alwaysOnTop: false,
    ios: [
      {
        id: "a0",
        key: "a",
        keySend: false,
        keyLong: false,
        keyMod: "none"
      },
      {
        id: "a1",
        key: "a",
        keySend: false,
        keyLong: false,
        keyMod: "none"
      },
      {
        id: "d2",
        key: "a",
        keySend: false,
        keyLong: false,
        keyMod: "none"
      },
      {
        id: "d3",
        key: "a",
        keySend: false,
        keyLong: false,
        keyMod: "none"
      },
      {
        id: "d4",
        key: "a",
        keySend: false,
        keyLong: false,
        keyMod: "none"
      },
      {
        id: "d5",
        key: "a",
        keySend: false,
        keyLong: false,
        keyMod: "none"
      }
    ],
    baudrates: [9600, 115200],
    keys: [
      "alt",
      "ctrl",
      "space",
      "shift",
      "enter",
      "space",
      "up",
      "down",
      "left",
      "right",
      "a",
      "b",
      "c",
      "d",
      "e",
      "f",
      "g",
      "h",
      "i",
      "j",
      "k",
      "l",
      "m",
      "n",
      "o",
      "p",
      "q",
      "r",
      "s",
      "t",
      "u",
      "v",
      "w",
      "x",
      "y",
      "z"
    ],
    mods: ["none", "alt", "command", "control", "shift"]
  }
});
