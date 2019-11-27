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
        pin: "a0",
        inMin: 0,
        inMax: 1023,
        outMin: 0,
        outMax: 127,
        keyThreshold: 64
      },
      {
        pin: "a1",
        inMin: 0,
        inMax: 1023,
        outMin: 0,
        outMax: 127,
        keyThreshold: 64
      },
      {
        pin: "d2",
        inMin: 0,
        inMax: 1023,
        outMin: 0,
        outMax: 127,
        keyThreshold: 64
      },
      {
        pin: "d3",
        inMin: 0,
        inMax: 1023,
        outMin: 0,
        outMax: 127,
        keyThreshold: 64
      }
    ],
    baudrates: [9600, 115200],
    keys: [
      "none",
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
      "z",
      "alt",
      "ctrl",
      "space",
      "shift",
      "enter",
      "space",
      "up",
      "down",
      "left",
      "right"
    ],
    mods: ["none", "alt", "command", "control", "shift"]
  }
});
